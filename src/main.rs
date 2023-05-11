use tokio::io::{copy, split, AsyncRead, AsyncWrite};
use tokio::runtime::Runtime;
use tokio::net::{TcpListener, TcpStream};
use std::thread;
use std::process::{Command, Stdio};
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::prelude::*;
use std::env;
use std::sync::mpsc;
use std::io::Lines;
use std::io::BufReader;
use tokio_socks::{tcp::Socks5Stream};

#[derive(Clone, Default, Debug)]
struct PTConnetArgs {
    proxy_type : String,
    addr : String,
    user_name : String,
    password : String
}

#[derive(Serialize, Deserialize, Clone, Debug)]
struct Config {
    // Role: client|server
    //"role": "server",
    role : String,
    // Where to store PT state files
    // "state": ".",
    state : String,
    // For server, which address to forward
    // For client, which address to listen
    //"local": "127.0.0.1:1080",
    local : String,
    // For server, which address to listen
    // For client, which address to connect
    //"server": "0.0.0.0:23456",
    server : String,
    // The PT command line
    //"ptexec": "obfs4proxy -logLevel=ERROR -enableLogging=true",
    ptexec: String,
    // The PT name, must be only one
    //"ptname": "obfs4",
    ptname: String,
    // [Client] PT arguments
    //"ptargs": "cert=AAAAAAAAAAAAAAAAAAAAAAAAAAAAA+AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA;iat-mode=0",
    #[serde(default)]
    ptargs : String,
    // [Optional][Server] PT options
    // <key>=<value> [;<key>=<value> ...]
    // "ptserveropt": "",
    #[serde(default)]
    ptserveropt: String,
    // [Optional][Client] Which outgoing proxy must PT use
    // <proxy_type>://[<user_name>][:<password>][@]<ip>:<port>
    //"ptproxy": ""
    #[serde(default)]
    ptproxy: String,
}

async fn parse_lines<R:std::io::BufRead>(lines : &mut Lines<R>, cfg: &Config, tx: mpsc::Sender<PTConnetArgs>) {
    while let Ok(line) = lines.next().unwrap() {
        // split into words, and check the first one
        let line_clone = line.clone();
        let mut sp = line_clone.split_whitespace();
        let kw = sp.next().unwrap();
        let next = sp.next();
        if ["ENV-ERROR", "VERSION-ERROR", "PROXY-ERROR", "CMETHOD-ERROR", "SMETHOD-ERROR"].contains(&kw) {
            panic!("PT returned error: {}", line)
        } else if kw == "VERSION" &&  next != Some("1") {
            panic!("PT returned invalid version: {:?}", next)
        } else if kw == "PROXY" && next != Some("DONE") {
            panic!("PT returned invalid info:  {}", line)
        } else if kw == "CMETHOD" {
            let ptname = next.unwrap();
            if ptname == cfg.ptname {
                let pttype = sp.next().unwrap();
                let ptaddr = sp.next().unwrap();
                let _ = tx.send(PTConnetArgs { 
                    proxy_type: pttype.to_uppercase(), 
                    addr: ptaddr.into(), 
                    user_name: cfg.ptargs.clone(), 
                    password: "\0".into() });
            }
        } else if kw == "SMETHOD" {
            let ptname = next.unwrap();
            if ptname == cfg.ptname {
                println!("===== Server information =====");
                println!("{}", line.clone().replace(",", ";"));
                println!("===== Server information end =====");
            }
        } else if (kw == "CMETHODS" || kw == "SMETHODS") && next == Some("DONE") {
            println!("PT started successfully.");
            return
        } else {
            println!("{}", line)
        }
    }
}

async fn run_pt(config: Config, tx: mpsc::Sender<PTConnetArgs>) -> Result<(), Box<dyn std::error::Error>> {
    let command_args = shell_words::split(config.ptexec.clone().as_str()).unwrap();
    let mut command = Command::new(command_args[0].clone());
    command.args(&command_args[1..]);
    command.env("TOR_PT_STATE_LOCATION", config.state.clone());
    command.env("TOR_PT_MANAGED_TRANSPORT_VER", "1");
    if config.role == "client" {
        command.env("TOR_PT_CLIENT_TRANSPORTS", config.ptname.clone());
        command.env("TOR_PT_PROXY", config.ptproxy.clone());
    } else {
        command.env("TOR_PT_SERVER_TRANSPORTS", config.ptname.clone());
        command.env("TOR_PT_SERVER_BINDADDR", format!("{}-{}", config.ptname, config.server));
        command.env("TOR_PT_ORPORT", config.local.clone());
        command.env("TOR_PT_EXTENDED_SERVER_PORT", "");
        if config.ptserveropt != "" {
            let opt = config.ptserveropt.split(";").map(|kv| {
                format!("{}:{}", config.ptname, kv)
            }).collect::<Vec<_>>().join(";");
            command.env("TOR_PT_SERVER_TRANSPORT_OPTIONS", opt);
        }
    }
    let mut child = command.stdout(Stdio::piped()).spawn().unwrap();
    println!("run ptexec");
    let out = child.stdout.take();
    if out.is_none() {
        panic!("PT failed to start.");
    }
    let mut lines = BufReader::new(out.unwrap()).lines();

    parse_lines(&mut lines, &config, tx).await;
    while let Ok(line) = lines.next().unwrap() {
        println!("PT: {}", line);
    }
    child.wait().unwrap();
    println!("PT exited.");
    Ok(())
}

async fn run_client(config: Config) -> Result<(), Box<dyn std::error::Error>> {
    let config_clone = config.clone();
    // create mpsc channel
    println!("start run client");
    let (tx, rx) = mpsc::channel();
    let tx_clone = tx.clone();
    thread::spawn(move || {
        let mut rt = Runtime::new().unwrap();
        rt.block_on(async move {
            let _ = run_pt(config_clone, tx_clone).await;
        });
    });
    let pt_args = rx.recv()?;
    println!("PT args: {:?}", pt_args);
    let listener = TcpListener::bind(config.local.clone()).await.unwrap();
    loop {
        let (stream, _) = listener.accept().await?;
        let pt_args_clone = pt_args.clone();
        let cfg_clone = config.clone();
        tokio::spawn(async move {
            handle_connection(stream, pt_args_clone, &cfg_clone).await.unwrap();
        });
    }
}


#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();
    let file_name= &args[1];
    let mut file = File::open(file_name).unwrap();
    let mut contents = String::new();
    file.read_to_string(&mut contents).unwrap();

    let config: Config = serde_json::from_str(&contents).unwrap();
    println!("config {:?}", config);
    if config.role == "client" {
        run_client(config).await.unwrap();
    } else {
        let (tx, rx) = mpsc::channel();
        run_pt(config, tx).await.unwrap()
    }
    Ok(())
}

async fn handle_connection(stream: TcpStream, pt_args : PTConnetArgs, cfg : &Config) -> Result<(), Box<dyn std::error::Error>> {
    // handle incoming traffic
    // TODO ignore the pt_type, now we only deal with socks5
    let (mut reader, mut writer) = split(stream);
    let proxy_stream = TcpStream::connect(pt_args.addr).await?;
    let  remote = Socks5Stream::connect_with_password_and_socket(proxy_stream, cfg.server.clone(), &pt_args.user_name, &pt_args.password).await?;
    // proxy steam with remote
    let (mut remote_reader, mut remote_writer) = split(remote);
    // create pipeline
    let client_to_server = copy(&mut reader, &mut remote_writer);
    let server_to_client = copy(&mut remote_reader, &mut writer);
    let (client_to_server_result, server_to_client_result) = tokio::join!(client_to_server, server_to_client);
    // Handle any errors
    client_to_server_result?;
    server_to_client_result?;
    Ok(())
}
