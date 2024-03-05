mod farm;
mod providers;
mod proxy_table;

use crate::{
    providers::{webshare::WebshareParser, ProxyFileParser},
    proxy_table::ProxyTable,
};

use std::{fs, str::FromStr, thread};

use farm::gcp::{
    config::config_firewall,
    create::create_vms,
    delete::{delete_vms, query_and_delete_vms},
    query::query_vms,
};
use locust_core::{
    crud::proxies::{
        add_proxies, delete_proxies_by_ids, delete_proxies_by_tags, get_proxies_by_tags,
    },
    get_conn_string, new_pool,
};

use clap::{Parser, Subcommand, ValueEnum};
use providers::infatica::InfaticaParser;
use refinery::config::Config;

mod embedded {
    use refinery::embed_migrations;
    embed_migrations!("../migrations");
}

#[derive(Debug, Parser)]
#[command(arg_required_else_help = true)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Debug, Clone, Subcommand)]
enum Command {
    /// A sumbcommand for configuring proxies
    Configure {
        #[command(subcommand)]
        command: ConfigureCommand,
    },
    /// A subcommand for importing proxies
    Import {
        file: String,

        #[arg(short, long)]
        provider: ProxyProvider,
    },
    /// A subquery for querying existing proxies
    Query {
        /// Comma separated tags for which to subset the proxies
        #[arg(short, long, value_delimiter = ',')]
        tags: Vec<String>,
    },
    /// A subcommand for managing proxy farms
    Farm {
        #[command(subcommand)]
        command: FarmCommand,

        #[arg(short, long)]
        project: String,

        #[arg(short, long, default_value_t = String::from("us-central1-a"))]
        zone: String,
    },
    Migrate {},
}

#[derive(Debug, Clone, Subcommand)]
enum ConfigureCommand {
    Domain {
        host: String,

        #[command(subcommand)]
        command: ConfigureDomainCmd,
    },
    Firewall {},
}

#[derive(Debug, Clone, Subcommand)]
enum ConfigureDomainCmd {
    Tags {
        #[arg(short, long, value_delimiter = ',')]
        tags: Vec<String>,

        #[arg(short, long, default_value_t = false)]
        remove: bool,
    },
}

#[derive(Debug, Clone, Subcommand)]
enum FarmCommand {
    Create {
        #[arg(short, long, default_value_t = 5)]
        num: u16,

        #[arg(short, long)]
        username: String,

        #[arg(long)]
        pwd: String,
    },
    Delete {
        #[arg(short, long, default_value_t = String::from("us-central1-a"))]
        zone: String,
    },
    Cycle {
        #[arg(short, long, default_value_t = 5)]
        num: u16,

        #[arg(short, long)]
        username: String,

        #[arg(long)]
        pwd: String,

        #[arg(short, long, default_value_t = String::from("us-central1-a"))]
        zone: String,
    },
}

#[derive(Debug, Clone, ValueEnum)]
enum ProxyProvider {
    Webshare,
    Infatica,
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();
    let db_pool = new_pool().await.expect("error creating db pool");
    match cli.command {
        Command::Configure { command } => match command {
            ConfigureCommand::Domain { host, command } => match command {
                ConfigureDomainCmd::Tags { tags, remove } => {
                    println!("{} {:?}, {}", host, tags, remove);
                }
            },
            ConfigureCommand::Firewall {} => {
                config_firewall();
            }
        },
        Command::Import { file, provider } => {
            let content = fs::read_to_string(file).expect("error reading import file");
            match provider {
                ProxyProvider::Webshare => {
                    let parser = WebshareParser {};
                    let proxies = parser.parse_file(&content);
                    let n_proxies = proxies.len();

                    let tags = vec!["webshare"];
                    add_proxies(&db_pool, &proxies, &tags)
                        .await
                        .expect("error adding proxies");
                    println!("Successfully added {} proxies!", n_proxies);
                }
                ProxyProvider::Infatica => {
                    let parser = InfaticaParser {};
                    let proxies = parser.parse_file(&content);
                    let n_proxies = proxies.len();

                    let tags = vec!["infatica"];
                    add_proxies(&db_pool, &proxies, &tags)
                        .await
                        .expect("error adding proxies");
                    println!("Successfully added {} proxies!", n_proxies);
                }
            }
        }
        Command::Query { tags } => {
            let tags: Vec<&str> = tags.iter().map(AsRef::as_ref).collect();
            let proxies = get_proxies_by_tags(&db_pool, &tags)
                .await
                .expect("error fetching proxies");
            let table = ProxyTable(proxies);
            println!("{}", table);
        }
        Command::Farm {
            command,
            project,
            zone,
        } => match command {
            FarmCommand::Create { num, username, pwd } => {
                let vms = create_vms(&project, &zone, &username, &pwd, num);

                let tags = vec!["squid"];
                add_proxies(&db_pool, &vms, &tags)
                    .await
                    .expect("error adding proxies");
                println!("Successfully added {} proxies!", vms.len());
            }
            FarmCommand::Delete { zone } => {
                query_and_delete_vms(zone);
                let tags = vec!["squid"];
                delete_proxies_by_tags(&db_pool, &tags)
                    .await
                    .expect("error deleting proxies from db");
                println!("Done!");
            }
            FarmCommand::Cycle {
                num,
                username,
                pwd,
                zone,
            } => {
                // We add new vms and delete from the db
                // before deleting the old vms to minimize
                // downtime. If you dont do this youd either have
                // no proxies or hitting vms that dont exist anymore

                let tags = vec!["squid"];

                // Get existing vm proxies
                let old_vms = query_vms(&zone);
                let old_proxies = get_proxies_by_tags(&db_pool, &tags)
                    .await
                    .expect("error getting old proxies from db");

                // Create new VMs
                let new_vms = create_vms(&project, &zone, &username, &pwd, num);

                // Add new proxies
                add_proxies(&db_pool, &new_vms, &tags)
                    .await
                    .expect("error adding proxies");

                // Delete old proxies from DB
                let ids_delete: Vec<i32> = old_proxies.iter().map(|p| p.id).collect();
                delete_proxies_by_ids(&db_pool, &ids_delete)
                    .await
                    .expect("error deleting old proxies");

                // Delete old vms
                delete_vms(zone, old_vms);

                println!("Successfully added {} proxies!", new_vms.len());
                println!("Done!");
            }
        },
        Command::Migrate {} => {
            let conn_string = get_conn_string();
            let mut conf = Config::from_str(&conn_string).expect("Invalid connection string");
            thread::spawn(move || {
                embedded::migrations::runner().run(&mut conf).unwrap();
            })
            .join()
            .expect("Could not run migration thread");
            println!("Migrations applied!");
        }
    }
}
