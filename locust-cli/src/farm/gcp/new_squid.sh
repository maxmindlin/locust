#!/bin/bash

project=$1
zone=$2
vm_tag='squid-proxy-locust'
username=$3
password=$4
n=$5
id=$6
vm_name="squid-$id-$n"

# create vm and startup script (metadata)
gcloud compute instances create $vm_name \
	--format="csv(name,zone,status,networkInterfaces[0].accessConfigs[0].natIP:label=external_ip)" \
	--project=$project \
	--zone=$zone \
	--machine-type=e2-micro \
	--image-family=debian-12 \
	--image-project=debian-cloud \
	--tags=$vm_tag \
	--metadata=startup-script="#! /bin/bash
    sudo apt-get update
    sudo apt-get install -y squid apache2-utils
    cat <<'EOF' > /etc/squid/squid.conf
    acl Safe_ports port 80 443
    acl CONNECT method CONNECT 
    http_access allow localhost
    http_access deny !Safe_ports
    http_access deny CONNECT !Safe_ports
    forwarded_for off
    request_header_access Allow allow all
    request_header_access Authorization allow all
    request_header_access WWW-Authenticate allow all
    request_header_access Proxy-Authorization allow all
    request_header_access Proxy-Authenticate allow all
    request_header_access Cache-Control allow all
    request_header_access Content-Encoding allow all
    request_header_access Content-Length allow all
    request_header_access Content-Type allow all
    request_header_access Date allow all
    request_header_access Expires allow all
    request_header_access Host allow all
    request_header_access If-Modified-Since allow all
    request_header_access Last-Modified allow all
    request_header_access Location allow all
    request_header_access Pragma allow all
    request_header_access Accept allow all
    request_header_access Accept-Charset allow all
    request_header_access Accept-Encoding allow all
    request_header_access Accept-Language allow all
    request_header_access Content-Language allow all
    request_header_access Mime-Version allow all
    request_header_access Retry-After allow all
    request_header_access Title allow all
    request_header_access Connection allow all
    request_header_access Proxy-Connection allow all
    request_header_access User-Agent allow all
    request_header_access Cookie allow all
    request_header_access All deny all
    auth_param basic program /usr/lib/squid/basic_ncsa_auth /etc/squid/passwd
    auth_param basic realm proxy
    acl authenticated proxy_auth REQUIRED
    http_access allow authenticated
    http_port 3128
    EOF
    echo \"$username:$(openssl passwd -apr1 $password)\" | sudo tee /etc/squid/passwd > /dev/null
    sudo systemctl restart squid
    "
