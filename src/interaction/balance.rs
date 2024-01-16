use sparkle_convenience::{error::IntoError, reply::Reply};
use twilight_interactions::command::{CommandModel, CreateCommand};
use twilight_util::builder::embed::EmbedBuilder;

use crate::api::get_user_data;

use super::InteractionContext;

#[derive(CreateCommand, CommandModel, Debug)]
#[command(name = "balance", desc = "see your balance")]
pub struct BalanceCommand {}

impl InteractionContext<'_> {
    pub async fn handle_balance_command(self) -> Result<(), anyhow::Error> {
        let user = self.interaction.author().ok()?;

        let user_data = get_user_data(user.id).await;

        match user_data {
            Err(_) => {
                let error_embed = EmbedBuilder::new()
                    .title("Error")
                    .color(self.ctx.config.error_color)
                    .description("The user you tried to request was not found in our system.")
                    .validate()?
                    .build();

                self.handle
                    .reply(Reply::new().embed(error_embed).ephemeral())
                    .await?;
            }
            Ok(data) => {
                let balance_embed = EmbedBuilder::new()
                    .title("Success")
                    .color(self.ctx.config.success_color)
                    .description(format!(
                        "Your balance is `${:.2} USD`",
                        (data.balance as f32) / 100.00
                    ))
                    .validate()?
                    .build();

                self.handle
                    .reply(Reply::new().embed(balance_embed).ephemeral())
                    .await?;
            }
        }

        Ok(())
    }
}
