use std::any::Any;

use twilight_interactions::command::{CommandModel, CreateCommand};
use sparkle_convenience::reply::Reply;
use twilight_util::builder::embed::EmbedBuilder;

use super::InteractionContext;

#[derive(CommandModel, CreateCommand)]
#[command(name = "adminbal", desc = "Get the total bot balance")]
pub struct Command;

impl InteractionContext<'_> {
    pub async fn handle_adminbal_command(self) -> Result<(), anyhow::Error> {
        let balance_request = self.ctx.sms.clone().get_api_balance().await;
        println!("{:?}", balance_request.type_id());
        match balance_request {
            Some(balance) => {
                let embed = EmbedBuilder::new().title("Success").description(format!("Your api balance is: ${:.2}", balance)).validate()?.build();
                self.handle.reply(Reply::new().content("Helped").embed(embed)).await?;
            }
            None => {
                let embed = EmbedBuilder::new().title("Error").description("Error getting balance").color(0xff0000).validate()?.build();
                self.handle.reply(Reply::new().embed(embed).ephemeral()).await?;
                return Err(anyhow::anyhow!("Error"));
            }
        }
        Ok(())
    }
}