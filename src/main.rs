use std::error::Error;

use clap::{arg, command};
use get_cookies::read_cookie_with_title;
use reqwest::{
    header::{self, HeaderValue, COOKIE},
    Client,
};

mod models;
use models::RunningResponse;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
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
        cookie = Some(fetch_cookie_from_browser().await?)
    }

    let cookie = cookie.unwrap();
    print_vm_status(&cookie).await?;

    Ok(())
}

async fn fetch_cookie_from_browser() -> Result<String, Box<dyn Error>> {
    let url = "https://tryhackme.com/dashboard";
    let title = "Please log into TryHackMe";
    let cookie = read_cookie_with_title(
        &url,
        |cookie_str: &String| cookie_str.contains("_cioid"),
        title,
    )
    .await?;
    Ok(cookie)
}

fn create_client_with_cookie(cookie: &str) -> Result<Client, Box<dyn Error>> {
    let mut headers = header::HeaderMap::new();
    headers.insert(COOKIE, HeaderValue::from_str(&cookie).unwrap()); // this is much more difficult than it needs to be
    let client = reqwest::Client::builder()
        .default_headers(headers)
        // .proxy(reqwest::Proxy::all("http://127.0.0.1:8080")?)
        // .danger_accept_invalid_certs(true)
        .build()?;

    Ok(client)
}

async fn print_vm_status(cookie: &str) -> Result<(), Box<dyn Error>> {
    let client = create_client_with_cookie(cookie)?;
    let url = "https://tryhackme.com/api/v2/vms/running";
    let running = client
        .get(url)
        .send()
        .await?
        .json::<RunningResponse>()
        .await?;

    if running.status != "success" {
        match running.message {
            Some(message) => eprintln!("failed to request VM status: {message}"),
            _ => eprintln!("failed to request VM status - no reason given"),
        }
        return Ok(());
    }

    let data = running.data.unwrap();

    if data.len() == 0 {
        println!("no vms running");
        return Ok(());
    }

    for vm in data {
        println!(
            "title:\t\t{}\nexpires in:\t{} minutes",
            vm.title,
            vm.minutes_remaining()
        );
        match (vm.remote.private_ip, vm.credentials) {
            (Some(private_ip), Some(credentials)) => println!(
                "internal ip:\t{}\n  public ip:\t{}\n  username:\t{}\n  password:\t{}\n",
                private_ip, vm.internal_ip, credentials.username, credentials.password
            ),
            _ => println!("internal ip:\t{}\n", vm.internal_ip),
        }
    }

    Ok(())
}
