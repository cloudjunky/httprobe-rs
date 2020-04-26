use std::io;
use reqwest::{Response, Client, Url};
use tokio;
use std::time::Duration;
use clap::{Arg, App};
use std::fs::File;
use std::io::{BufRead, BufReader};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let matches = App::new("httprobe-rs")
                            .version("1.0")
                            .about("Reads from STDIN and GETs the URL.")
                            .arg(Arg::with_name("INPUT")
                                .help("Sets the input file to use")
                                .required(true)
                                .index(1))
                            .arg(Arg::with_name("t")
                                .short("t")
                                .long("timeout")
                                .multiple(false)
                                .takes_value(true)
                                .help("Sets timeout in seconds."))
                            .get_matches();
    
    //Default timeout is 3 seconds.
    let default_timeout = "3";
    let timeout_arg = matches.value_of("t").unwrap_or(default_timeout);

    //Parse the timeout string but if it doesn't error.
    let timer = timeout_arg.parse::<u64>().expect("Couldn't parse timeout value.");

    //Setup the Client for how it will query.
    let timeout = Duration::new(timer,0);
    let client = Client::builder()
        .timeout(timeout)
        .build()?;

    let input_value = matches.value_of(&"INPUT").unwrap();

    if input_value  == "-" {
        process_stdin(&client).await?;
    } else {
        process_file(&client, input_value).await?;
    }

    Ok(())
}

async fn process_stdin(client: &Client) -> Result<(),Box<dyn std::error::Error>>  {
    loop {
        let mut url = String::new();
        match io::stdin().read_line(&mut url) {
            Ok(len) => if len == 0 {
                break;
            } else {
                let result = probe_site(&client, &url).await;
                match result {
                    Ok(r) => println!("{} {}", r.url(), r.status()),
                    Err(_) => (),
                }
            }
            Err(error) => {
                eprintln!("Error: {}", error);
            }
        }
    }
    Ok(())
}

async fn process_file(client: &Client, filename: &str) -> Result<(),Box<dyn std::error::Error>> {
    let f = File::open(filename).unwrap();
    let f = BufReader::new(f);
    for url in f.lines() {
        let url = url.expect("No Line");
        let result = probe_site(&client, &url).await;
        match result {
            Ok(r) => println!("{} {}", r.url(), r.status()),
            Err(_) => (),
        }
    }
    Ok(())
}

async fn probe_site(client: &Client, url: &str) -> Result<Response, Box<dyn std::error::Error>> {
    let parsed_url = Url::parse(url)?;
    let res = client.get(parsed_url.as_str()).send().await?;
    Ok(res)
}
