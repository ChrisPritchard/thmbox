use std::{error::Error, fmt::Display, result};

use clap::{arg, command};
use get_cookies::read_cookie_with_title;
use reqwest::{header::{self, HeaderValue, COOKIE}, Client};

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
        cookie = Some(fetch_cookie_from_browser(None).await?)
    }

    let cookie = cookie.unwrap();
    let result = print_vm_status(&cookie).await;
    if let Err(e) = result {
        if let Some(_) = e.downcast_ref::<UnauthorizedError>() {
            let bad_cookie = cookie.split(";").find(|s| s.starts_with("connect.sid=")).unwrap();
            println!("this cookie is invalid: {bad_cookie}");
            let cookie = fetch_cookie_from_browser(Some(&bad_cookie)).await?;
            println!("new cookie: {cookie}");
            print_vm_status(&cookie).await?
        } else {
            panic!("unknown error: {:?}", e);
        }
    }
    // match  {
    //     Ok(_) => (),
    //     Err(e) => {
    //         if let Some(_) =  {
    //             let bad_cookie = cookie.split(";").find(|s| s.starts_with("connect.sid=")).unwrap();
    //             let cookie = Some(fetch_cookie_from_browser(Some(&bad_cookie)).await?);
    //             let cookie = cookie.unwrap();
    //             print_vm_status(&cookie).await?
    //         } else {
    //             
    //         }
    //     }
    // }

    Ok(())
}

async fn fetch_cookie_from_browser(invalid_cookie_to_ignore: Option<&str>) -> Result<String, Box<dyn Error>> {
    let url = "https://tryhackme.com/dashboard";
    let title = "Please log into TryHackMe";
    let cookie = 
        if invalid_cookie_to_ignore.is_none() {
            read_cookie_with_title(&url, |cookie_str: &String| cookie_str.contains("_cioid"), title).await?
        } else {
            read_cookie_with_title(&url, |cookie_str: &String| cookie_str.contains("_cioid"), title).await?
        };
    Ok(cookie)
}

fn create_client_with_cookie(cookie: &str) -> Result<Client, Box<dyn Error>> {
    let mut headers = header::HeaderMap::new();
    headers.insert(COOKIE, HeaderValue::from_str(&cookie).unwrap()); // this is much more difficult than it needs to be
    let client = reqwest::Client::builder()
        .default_headers(headers)
        .proxy(reqwest::Proxy::all("http://127.0.0.1:8080")?)
        .danger_accept_invalid_certs(true)
        .build()?;

    Ok(client)
}

async fn print_vm_status(cookie: &str) -> Result<(), Box<dyn Error>> {
    let client = create_client_with_cookie(cookie)?;
    let url = "https://tryhackme.com/api/v2/vms/running";
    let running = client.get(url).send().await?.json::<RunningResponse>().await?;

    if running.status != "success" {
        match running.message {
            Some(s) if s == "Unauthorized" => {
                return Err(Box::new(UnauthorizedError{}));
            },
            Some(message) => eprintln!("failed to request VM status: {message}"),
            _ => eprintln!("failed to request VM status - no reason given")
        }
        return Ok(())
    }

    let data = running.data.unwrap();

    if data.len() == 0 {
        println!("no vms running");
        return Ok(());
    }

    for vm in data {
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

#[derive(Debug)]
struct UnauthorizedError {}

impl Error for UnauthorizedError {}

impl Display for UnauthorizedError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("unauthorized")
    }
}