mod farm;
mod providers;
mod proxy_table;

use crate::{
    providers::{webshare::WebshareParser, ProxyFileParser},
    proxy_table::ProxyTable,
};

use std::fs;

use farm::gcp::{create::create_vms, delete::delete_vms};
use locust_core::{
    crud::proxies::{add_proxies, delete_proxies_by_tags, get_proxies_by_tags},
    new_pool,
};

use clap::{Parser, Subcommand, ValueEnum};

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
}

#[derive(Debug, Clone, Subcommand)]
enum ConfigureCommand {
    Domain {
        host: String,

        #[command(subcommand)]
        command: ConfigureDomainCmd,
    },
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
            }
        }
        Command::Query { tags } => {
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
                let vms = create_vms(&project, &zone, &username, &pwd, num).await;

                let tags = vec!["squid"];
                add_proxies(&db_pool, &vms, &tags)
                    .await
                    .expect("error adding proxies");
                println!("Successfully added {} proxies!", vms.len());
            }
            FarmCommand::Delete { zone } => {
                delete_vms(zone).await;
                let tags = vec!["squid".to_string()];
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
                delete_vms(zone.clone()).await;
                let tags = vec!["squid".to_string()];
                delete_proxies_by_tags(&db_pool, &tags)
                    .await
                    .expect("error deleting proxies from db");
                let vms = create_vms(&project, &zone, &username, &pwd, num).await;

                let tags = vec!["squid"];
                add_proxies(&db_pool, &vms, &tags)
                    .await
                    .expect("error adding proxies");
                println!("Successfully added {} proxies!", vms.len());
                println!("Done!");
            }
        },
    }
}
