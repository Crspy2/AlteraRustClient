use sparkle_convenience::reply::Reply;
use twilight_interactions::command::{CommandModel, CreateCommand};
use twilight_util::builder::embed::EmbedBuilder;

use super::InteractionContext;

#[derive(CommandModel, CreateCommand)]
#[command(name = "adminbal", desc = "Get the total bot balance")]
pub struct AdminBalCommand;

impl InteractionContext<'_> {
    pub async fn handle_adminbal_command(self) -> Result<(), anyhow::Error> {
        let balance_request = self.ctx.sms.clone().get_api_balance().await;
        match balance_request {
            Ok(balance) => {
                let embed = EmbedBuilder::new()
                    .title("Success")
                    .description(format!("The total balance of the bot is `${:.2}`", balance))
                    .color(self.ctx.config.success_color)
                    .validate()?
                    .build();
                self.handle
                    .reply(Reply::new().embed(embed).ephemeral())
                    .await?;
            }
            Err(err) => {
                let embed = EmbedBuilder::new()
                    .title("Error")
                    .description(err.errors.iter().next().unwrap().message.as_str())
                    .color(self.ctx.config.error_color)
                    .validate()?
                    .build();
                self.handle
                    .reply(Reply::new().embed(embed).ephemeral())
                    .await?;
            }
        }
        Ok(())
    }
}
