use futures::StreamExt;
use sms::SmsClient;
use sparkle_convenience::Bot;
use std::{env, fmt::Debug, sync::Arc};
use twilight_gateway::{error::ReceiveMessageErrorType, stream::ShardEventStream, EventTypeFlags};
use twilight_http as _;
use twilight_model::{
    application::interaction::Interaction,
    gateway::{event::Event, Intents},
};

mod interaction;
mod logic;
mod sms;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("unknown interaction: {0:#?}")]
    UnknownInteraction(Interaction),
}

#[derive(Debug)]
struct Config {
    debug_scope: u64,
    price_multiplier: i8,
    success_color: u32,
    error_color: u32,
}

#[derive(Debug)]
struct Context {
    bot: Bot,
    config: Config,
    sms: SmsClient,
}

impl Context {
    async fn handle_event(&self, event: Event) {
        match event {
            Event::InteractionCreate(interaction) => self.handle_interaction(interaction.0).await,
            _ => {}
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    dotenvy::dotenv()?;
    tracing_subscriber::fmt().pretty().init();

    let (bot, mut shards) = Bot::new(
        env::var("DISCORD_TOKEN")?,
        Intents::empty(),
        EventTypeFlags::INTERACTION_CREATE,
    )
    .await?;

    tracing::info!("Connected as {}", bot.user.name);

    let config = Config {
        debug_scope: env::var("DEBUG_SCOPE")?.parse().unwrap(),
        success_color: 0x65C97A,
        error_color: 0xE85041,
        price_multiplier: 10,
    };
    let sms = SmsClient::new(env::var("API_KEY")?);

    let ctx = Arc::new(Context { bot, sms, config });

    ctx.create_commands().await.unwrap_or_else(|err| {
        tracing::error!("Failed to create commands:\n{}", err.backtrace());
    });

    let mut events = ShardEventStream::new(shards.iter_mut());
    while let Some((_, event_res)) = events.next().await {
        let ctx_event_ref = Arc::clone(&ctx);
        match event_res {
            Ok(event) => {
                tokio::spawn(async move {
                    ctx_event_ref.handle_event(event).await;
                });
            }
            Err(err)
                if !matches!(
                    err.kind(),
                    ReceiveMessageErrorType::Deserializing { .. } | ReceiveMessageErrorType::Io
                ) =>
            {
                ctx_event_ref.bot.log(&err).await;

                if err.is_fatal() {
                    break;
                }
            }
            Err(_) => {}
        };
    }

    Ok(())
}
