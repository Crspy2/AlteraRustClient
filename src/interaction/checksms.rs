use crate::api::{get_user_data, update_user_balance};
use sparkle_convenience::reply::Reply;
use twilight_interactions::command::{CommandModel, CreateCommand};
use twilight_util::builder::embed::{EmbedBuilder, EmbedFieldBuilder};

use super::InteractionContext;

#[derive(CreateCommand, CommandModel, Debug)]
#[command(
    name = "checksms",
    desc = "check for an incoming sms code.",
    dm_permission = false
)]
pub struct CheckSMSCommand {}

impl InteractionContext<'_> {
    pub async fn handle_checksms_command(self) -> Result<(), anyhow::Error> {
        let user_data_request = get_user_data(self.interaction.author_id().unwrap()).await;
        if let Err(_) = user_data_request {
            let no_user_embed = EmbedBuilder::new()
                .title("Error")
                .color(self.ctx.config.error_color)
                .description(format!(
                    "No user could be found for **@{}**",
                    self.interaction.author().unwrap().name
                ))
                .validate()?
                .build();

            self.handle
                .reply(Reply::new().embed(no_user_embed).ephemeral())
                .await?;

            return Ok(());
        }

        let user_data = user_data_request.unwrap();
        let user_number = user_data.numbers.into_iter().rev().into_iter().nth(0);

        match user_number {
            None => {
                let no_number_embed = EmbedBuilder::new()
                    .title("Error")
                    .color(self.ctx.config.error_color)
                    .description(format!(
                        "No number could be found for **@{}**",
                        self.interaction.author().unwrap().name
                    ))
                    .validate()?
                    .build();

                self.handle
                    .reply(Reply::new().embed(no_number_embed).ephemeral())
                    .await?;

                return Ok(());
            }
            Some(number) => {
                if user_data.balance < number.price {
                    let no_balance_embed = EmbedBuilder::new()
                        .title("Error")
                        .color(self.ctx.config.error_color)
                        .description(format!("You don't have enough balance to check for an sms code. Your balance is **{}** and the price for this number is **{}**", user_data.balance, number.price))
                        .validate()?
                        .build();

                    self.handle
                        .reply(Reply::new().embed(no_balance_embed).ephemeral())
                        .await?;

                    return Ok(());
                }

                let sms_code_request = self.ctx.sms.clone().get_sms_code(&number.order_id).await;

                match sms_code_request {
                    Err(err) => {
                        tracing::error!("{:#?}", err);

                        let sms_code_embed = EmbedBuilder::new()
                            .title("Error")
                            .color(self.ctx.config.error_color)
                            .description(format!("An error occured while trying to get the sms code for **{}**. Please try again later.", number.number))
                            .validate()?
                            .build();

                        self.handle
                            .reply(Reply::new().embed(sms_code_embed).ephemeral())
                            .await?;

                        return Ok(());
                    }
                    Ok(sms_code) => {
                        match sms_code.status {
                            1 | 2 | 4 => {
                                let no_incomming_embed = EmbedBuilder::new()
                                    .title("Pending")
                                    .color(self.ctx.config.success_color)
                                    .description(format!("Incoming texts to +{}:\n```glsl\nMessage Not Received\n```", number.number))
                                    .field(EmbedFieldBuilder::new("Still Waiting?", "If you requested an SMS code and you have not 
                                            received it, remember it can take up to 5 minutes to receive the SMS code. You will not be
                                            charged until you receive a message"))
                                    .field(EmbedFieldBuilder::new("Expiration:", format!("<t:{}:R>", sms_code.expiration)))
                                    .validate()?
                                    .build();

                                self.handle
                                    .reply(Reply::new().embed(no_incomming_embed).ephemeral())
                                    .await?;
                            }
                            3 => {
                                let balance_removal_success = update_user_balance(
                                    user_data.id,
                                    user_data.balance - number.price,
                                )
                                .await;

                                if let Err(err) = balance_removal_success {
                                    tracing::error!("{:#?}", err);

                                    let sms_code_embed = EmbedBuilder::new()
                                        .title("Error")
                                        .color(self.ctx.config.error_color)
                                        .description(format!("An error occured while processing your request. Please try again later."))
                                        .validate()?
                                        .build();

                                    self.handle
                                        .reply(Reply::new().embed(sms_code_embed).ephemeral())
                                        .await?;

                                    return Ok(());
                                }

                                let log_embed = EmbedBuilder::new()
                                    .title("2fa Code Received")
                                    .color(self.ctx.config.success_color)
                                    .description(format!(
                                        "**@{}** has just received a 2fa code.",
                                        self.interaction.author().unwrap().name
                                    ))
                                    .field(EmbedFieldBuilder::new("Service:", number.service))
                                    .field(EmbedFieldBuilder::new("Country:", number.country))
                                    .validate()?
                                    .build();

                                let _ = self
                                    .ctx
                                    .bot
                                    .http
                                    .create_message(self.ctx.config.log_channel)
                                    .embeds(&vec![log_embed]);

                                let sms_embed = EmbedBuilder::new()
                                    .title("Success")
                                    .color(self.ctx.config.success_color)
                                    .description(format!(
                                        "Incoming texts to +{}:\n```glsl\n{}\n```",
                                        number.number,
                                        sms_code.full_sms.unwrap()
                                    ))
                                    .field(
                                        EmbedFieldBuilder::new("SMS Code:", sms_code.sms.unwrap())
                                            .inline(),
                                    )
                                    .field(
                                        EmbedFieldBuilder::new(
                                            "Expires:",
                                            format!("<t:{}:R>", sms_code.expiration),
                                        )
                                        .inline(),
                                    )
                                    .validate()?
                                    .build();

                                self.handle
                                    .reply(Reply::new().embed(sms_embed).ephemeral())
                                    .await?;
                            }

                            _ => {
                                let expired_embed = EmbedBuilder::new()
                                    .title("Error")
                                    .color(self.ctx.config.error_color)
                                    .description(
                                        "Your number has expired. Please generate a new one",
                                    )
                                    .validate()?
                                    .build();

                                self.handle
                                    .reply(Reply::new().embed(expired_embed).ephemeral())
                                    .await?;
                            }
                        };
                        return Ok(());
                    }
                }
            }
        }
    }
}
