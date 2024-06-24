use clap::{arg, command};
use get_cookies::read_cookie_with_title;

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
    let has_keys = cookie.clone().unwrap().contains("connect.sid=");
    if !has_keys {
        println!("invalid cookie specified (missing required keys)\nspawning browser")
    }

    if cookie.is_none() || !has_keys {
        let from_site = read_cookie_with_title("https://tryhackme.com/dashboard", |cookie_str: &String| {
                 cookie_str.contains("connect.sid=")
        }, "Please log into TryHackMe").await?;
        cookie = Some(from_site);
    }

    println!("{:?}", cookie);

    // let cookie = 

    // println!("Cookie header string: Cookie: {:?}", cookie);

    Ok(())
}
