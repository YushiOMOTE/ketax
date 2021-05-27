use anyhow::Result;
use log::*;
use structopt::StructOpt;

mod db;
mod graphql;
mod web;

use crate::db::Db;

#[derive(StructOpt, Debug)]
struct Opt {
    /// Size limit for json payload in KB
    #[structopt(short = "l", long = "limit", default_value = "4096", env)]
    limit_kb: usize,
    /// Directory path for sled database to store data.
    #[structopt(short = "d", long = "db", default_value = "/tmp/ketadb", env)]
    db: String,
    /// Path to address for the web server to bind.
    #[structopt(short = "b", long = "bind", default_value = "127.0.0.1:8888", env)]
    addr: String,
}

async fn run() -> Result<()> {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();

    let opt = Opt::from_args();
    info!("Start {:?}", opt);

    let db = Db::new(&opt.db)?;
    web::run(&opt.addr, db, opt.limit_kb).await?;

    Ok(())
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    run()
        .await
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, format!("{:?}", e)))
}
