//~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
//         i must start somewhere
//           very early prototype
//            will likely rewrite
//~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~


use std::process::Command;
use systemstat::{saturating_sub_bytes, Platform, System};

use std::thread;
use std::time::Duration;

use bollard::Docker;

use std::fs;

use serenity::async_trait;
use serenity::model::channel::Message;
use serenity::model::gateway::Ready;
use serenity::prelude::*;
use serenity::utils::MessageBuilder;

struct Handler;

static BACKUP_LOG: &str = "/your/backup/log.txt";
static BACKUP_SCRIPT: &str = "/your/backup/script.sh";

#[async_trait]
impl EventHandler for Handler {
    async fn message(&self, ctx: Context, msg: Message) {
        if msg.content == "!hwinfo" {
            // https://github.com/valpackett/systemstat/tree/trunk?tab=readme-ov-file
            // see systemstat crate for more stats
            // to implement: fan speed, gpu stats
            // to fix: uptime should be in minutes

            let sys = System::new();

            let msg_uptime = match sys.uptime() {
                Ok(uptime) => format!("**Uptime:** {:?}", uptime),
                Err(x) => format!("**Uptime:** *error: {}*", x),
            };

            let msg_c_use = match sys.cpu_load_aggregate() {
                Ok(cpu) => {
                    thread::sleep(Duration::from_secs(1));
                    let cpu = cpu.done().unwrap();
                    let cpu_per = cpu.system * 100.0;
                    format!("**CPU load:** {:.2}%", cpu_per)
                }
                Err(x) => format!("**CPU load:** *error: {}*", x),
            };

            let msg_c_temp = match sys.cpu_temp() {
                Ok(cpu_temp) => format!("**CPU temp:** {}", cpu_temp),
                Err(x) => format!("**CPU temp:** *{}*", x),
            };

            let msg_mem: String = match sys.memory() {
                Ok(mem) => format!(
                    "**Memory:** {} / {} used",
                    saturating_sub_bytes(mem.total, mem.free),
                    mem.total
                ),
                Err(x) => format!("**Memory:** error: {}", x),
            };

            // for the console
            println!("{}", msg_uptime);
            println!("{}", msg_c_use);
            println!("{}", msg_c_temp);
            println!("{}", msg_mem);

            // for discord
            let response = MessageBuilder::new()
                .push_line(msg_uptime)
                .push_line(msg_c_use)
                .push_line(msg_c_temp)
                .push_line(msg_mem)
                .build();

            if let Err(why) = msg.channel_id.say(&ctx.http, &response).await {
                println!("Error sending message: {why:?}");
            }
        }

        if msg.content == "!docker" {
            // TODO make this print to discord. dont know how to add vectors of strings to a serenity message
            println!("PRINT DOCKER IMAGES HERE");
            let docker = Docker::connect_with_socket_defaults().unwrap();
            let running_containers = docker
                .list_containers(Some(bollard::container::ListContainersOptions::<String> {
                    all: false,
                    ..Default::default()
                }))
                .await
                .unwrap();

            for container in running_containers {
                println!("{}", container.names.unwrap()[0]);
            }
        }

        // print info from a file
        if msg.content == "!backup-stat" {
            let contents =
                fs::read_to_string(BACKUP_LOG).expect("Should have been able to read the file");

            println!("backup last performed on: {contents}");
            let response = MessageBuilder::new()
                .push("**backup last performed on:** ")
                .push_line(contents)
                .build();

            if let Err(why) = msg.channel_id.say(&ctx.http, &response).await {
                println!("Error sending message: {why:?}");
            }
        }

        // danger: its probably not super great for discord bots to run scripts
        if msg.content == "!backup-run" {
            println!("PRINT BACKUP RESPONSE HERE");
            let mut bak_cmd = Command::new("sh");
            bak_cmd.args([BACKUP_SCRIPT]); // add arguments
            bak_cmd.status().expect("something went wrong");
        }

        if msg.content == "!network" {
            // dont know how im going to do this
            println!("PRINT NETWORK DEVICES HERE");
        }
    }

    
    async fn ready(&self, _: Context, ready: Ready) {
        println!("{} is connected!", ready.user.name);
    }
}

#[tokio::main]
async fn main() {
    // Configure the client with your Discord bot token in the environment.
    let token: &str = "[YOUR-DISCORD-TOKEN]";
    // Set gateway intents, which decides what events the bot will be notified about
    let intents = GatewayIntents::GUILD_MESSAGES
        | GatewayIntents::DIRECT_MESSAGES
        | GatewayIntents::MESSAGE_CONTENT;

    // Create a new instance of the Client, logging in as a bot. This will automatically prepend
    // your bot token with "Bot ", which is a requirement by Discord for bot users.
    let mut client = Client::builder(&token, intents)
        .event_handler(Handler)
        .await
        .expect("Err creating client");

    // Finally, start a single shard, and start listening to events.
    //
    // Shards will automatically attempt to reconnect, and will perform exponential backoff until
    // it reconnects.
    if let Err(why) = client.start().await {
        println!("Client error: {why:?}");
    }
}
