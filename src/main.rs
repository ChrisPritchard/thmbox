use clap::{arg, command};
use get_cookies::read_cookie_with_title;
use reqwest::{header::{self, HeaderValue, COOKIE}, Client};

mod models;
use models::RunningResponse;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let matches = command!()
        .arg(
            arg!(
                -c --cookie <THM_COOKIE> "Required cookies for the THM API - if not provided a browser popup will be used"
            )
            .required(false),
        )
        .get_matches();

    let mut cookie = matches.get_one::<String>("cookie").map(|s| s.to_owned());
    let has_keys = cookie.is_some() && cookie.clone().unwrap().contains("connect.sid=");
    if !has_keys {
        println!("invalid cookie specified (missing required keys)\nspawning browser")
    }

    if cookie.is_none() || !has_keys {
        let from_site = read_cookie_with_title("https://tryhackme.com/dashboard", |cookie_str: &String| {
                 cookie_str.contains("connect.sid=")
        }, "Please log into TryHackMe").await?;
        cookie = Some(from_site);
    }

    let mut headers = header::HeaderMap::new();
    let cookie = cookie.unwrap();
    headers.insert(COOKIE, HeaderValue::from_str(&cookie).unwrap()); // this is much more difficult than it needs to be
    let client = reqwest::Client::builder()
        .default_headers(headers)
    // .proxy(reqwest::Proxy::all("http://127.0.0.1:8080")?)
    // .danger_accept_invalid_certs(true)
        .build()?;

    print_vm_status(&client).await?;

    Ok(())
}

async fn print_vm_status(client: &Client) -> Result<(), Box<dyn std::error::Error>> {
    let url = "https://tryhackme.com/api/v2/vms/running";
    let running = client.get(url).send().await?.json::<RunningResponse>().await?;

    if running.status != "success" {
        eprintln!("failed to request VM status :( likely auth error");
        return Ok(());
    }

    if running.data.len() == 0 {
        println!("no vms running");
        return Ok(());
    }

    for vm in running.data {
        println!("title:\t\t{}\nexpires in:\t{} minutes", vm.title, vm.minutes_remaining());
        match (vm.remote.private_ip, vm.credentials) {
            (Some(private_ip), Some(credentials)) => 
                 println!("internal ip:\t{}\n  public ip:\t{}\n  username:\t{}\n  password:\t{}\n", 
                    private_ip, vm.internal_ip, credentials.username, credentials.password),
            _ => println!("internal ip:\t{}\n", vm.internal_ip)
        }
    }

    Ok(())
}
