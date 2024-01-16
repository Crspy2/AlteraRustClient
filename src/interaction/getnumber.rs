use crate::api::{get_user_data, post_user_number};
use crate::logic::{find_similar_countries, find_similar_services};
use crate::sms::get_country_prices::CountryPriceInfo;
use sparkle_convenience::{
    error::IntoError, interaction::extract::InteractionDataExt, reply::Reply,
};
use twilight_interactions::command::{CommandModel, CreateCommand};
use twilight_util::builder::embed::{EmbedBuilder, EmbedFieldBuilder};

use super::InteractionContext;

#[derive(CreateCommand, CommandModel, Debug)]
#[command(
    name = "getnumber",
    desc = "get a number for a specified service",
    dm_permission = false
)]
pub struct GetNumberCommand {
    #[command(desc = "the service to use the number for", min_length = 2)]
    pub service: String,
    #[command(desc = "the country the number should be from", min_length = 2)]
    pub country: Option<String>,
}

impl InteractionContext<'_> {
    pub async fn handle_getnumber_command(self) -> Result<(), anyhow::Error> {
        let options = GetNumberCommand::from_interaction(
            self.interaction.data.clone().ok()?.command().ok()?.into(),
        )?;

        let service = options.service;
        let country = options.country.unwrap_or("gb".to_string());

        let country_prices = self
            .ctx
            .sms
            .clone()
            .get_country_prices(service.as_str())
            .await
            .unwrap_or([].to_vec());

        // If service is invalid and request for prices fails
        if country_prices.is_empty() {
            let sms_services = self.ctx.sms.clone().get_service_list().await.unwrap();
            if sms_services.is_empty() {
                let request_error_embed = EmbedBuilder::new()
                    .title("Error")
                    .color(self.ctx.config.error_color)
                    .description("An error occured when making your request.")
                    .validate()?
                    .build();

                self.handle
                    .reply(Reply::new().embed(request_error_embed).ephemeral())
                    .await?;

                tracing::error!("Unable to obtain service list");
                return Ok(());
            }

            let similar_services = find_similar_services(service.as_str(), &sms_services);

            if similar_services.is_empty() {
                let no_services_embed = EmbedBuilder::new()
                    .title("Error")
                    .color(self.ctx.config.error_color)
                    .description(format!(
                        "No services similar to **{}** could be found.",
                        service
                    ))
                    .validate()?
                    .build();

                self.handle
                    .reply(Reply::new().embed(no_services_embed).ephemeral())
                    .await?;

                return Ok(());
            }

            let mut embed_desc =
                "The service you provided was invalid. Here are some similar ones that you might be interested in:\n".to_string();

            for (i, s) in similar_services.iter().enumerate() {
                embed_desc += format!(
                    "**{}:** {} | `{}%`\n",
                    i + 1,
                    s.service_info.name,
                    s.similarity_score
                )
                .as_str();
            }

            let invalid_service_error = EmbedBuilder::new()
                .title("Error")
                .color(self.ctx.config.error_color)
                .description(embed_desc)
                .validate()?
                .build();
            self.handle
                .reply(Reply::new().embed(invalid_service_error).ephemeral())
                .await?;

            return Ok(());
        }

        let filtered_country: Option<CountryPriceInfo>;
        filtered_country = if country.len() <= 3 {
            country_prices
                .iter()
                .find(|c| c.iso.to_lowercase() == country.to_lowercase())
                .cloned()
        } else {
            country_prices
                .iter()
                .find(|c| c.name.to_lowercase() == country.to_lowercase())
                .cloned()
        };

        match filtered_country {
            None => {
                let supported_countries = country_prices.clone();
                let similar_countries =
                    find_similar_countries(country.as_str(), &supported_countries);

                if similar_countries.is_empty() {
                    let no_countries_embed = EmbedBuilder::new()
                        .title("Error")
                        .color(self.ctx.config.error_color)
                        .description(format!(
                            "No countries similar to **{}** could be found",
                            country
                        ))
                        .validate()?
                        .build();

                    self.handle
                        .reply(Reply::new().embed(no_countries_embed).ephemeral())
                        .await?;
                } else {
                    let mut embed_desc = format!(
                        "Out of `{}` countries supporting this product, `{}` matched your input of **{}**.\n\n",
                        supported_countries.len(),
                        similar_countries.len(),
                        country
                    );

                    for (i, c) in similar_countries.iter().enumerate() {
                        embed_desc += format!(
                            "**{}:** {} | `{}%`\n",
                            i + 1,
                            c.country_info.name,
                            c.similarity_score
                        )
                        .as_str();
                    }

                    let similar_countries_embed = EmbedBuilder::new()
                        .title("Success")
                        .color(self.ctx.config.success_color)
                        .description(embed_desc)
                        .validate()?
                        .build();

                    self.handle
                        .reply(Reply::new().embed(similar_countries_embed))
                        .await?;
                }
            }
            Some(country_price_info) => {
                let user_data_request = get_user_data(self.interaction.author_id().unwrap())
                    .await;

                if let Err(_) = user_data_request {
                    let no_user_embed = EmbedBuilder::new()
                        .title("Error")
                        .color(self.ctx.config.error_color)
                        .description(format!("No user could be found for **@{}**", self.interaction.author().unwrap().name))
                        .validate()?
                        .build();

                    self.handle.reply(Reply::new().embed(no_user_embed).ephemeral()).await?;

                    return Ok(())
                }

                let user_data = user_data_request.unwrap();

                if user_data.balance
                    < (country_price_info.price * 100.00 * self.ctx.config.price_multiplier) as i32
                {
                    let broke_embed = EmbedBuilder::new()
                        .title("Error")
                        .color(self.ctx.config.error_color)
                        .description("Your balance is too low to make this transaction")
                        .validate()?
                        .build();

                    self.handle
                        .reply(Reply::new().embed(broke_embed).ephemeral())
                        .await?;

                    return Ok(());
                }

                let number_info = self
                    .ctx
                    .sms
                    .clone()
                    .create_sms_order(&service, &country)
                    .await;

                match number_info {
                    Err(err) => {
                        tracing::error!("{:#?}", err);
                        let invalid_response_embed = EmbedBuilder::new()
                            .title("Error")
                            .color(self.ctx.config.error_color)
                            .description("We are currently out of stock of numbers from the country you tried to order. Please try a different country or try again later!")
                            .validate()?
                            .build();

                        self.handle
                            .reply(Reply::new().embed(invalid_response_embed).ephemeral())
                            .await?;
                    }
                    Ok(info) => {
                        let sms_number = post_user_number(
                            &info.number.to_string(),
                            &info.service,
                            &info.country,
                            (info.cost * 100.00 * self.ctx.config.price_multiplier) as i32,
                            &info.order_id,
                            user_data.id,
                        )
                        .await;

                        match sms_number {
                            Err(err) => {
                                tracing::error!("{:#?}", err);
                                let error_embed = EmbedBuilder::new()
                                    .title("Error")
                                    .color(self.ctx.config.error_color)
                                    .description("An error occurred while processing your request. Please try again later.")
                                    .validate()?
                                    .build();

                                self.handle
                                    .reply(Reply::new().embed(error_embed).ephemeral())
                                    .await?;
                            }
                            Ok(_) => {
                                let log_embed = EmbedBuilder::new()
                                    .title("Number Generated")
                                    .color(self.ctx.config.success_color)
                                    .description(format!(
                                        "**@{}** | `{}` has just generated a number",
                                        self.interaction.author().unwrap().name,
                                        self.interaction.author_id().unwrap()
                                    ))
                                    .field(
                                        EmbedFieldBuilder::new("Service:", &info.service).inline(),
                                    )
                                    .field(
                                        EmbedFieldBuilder::new(
                                            "Country:",
                                            format!(
                                                "{}  :flag_{}:",
                                                &info.country,
                                                &country_price_info.iso.to_lowercase()
                                            ),
                                        )
                                        .inline(),
                                    )
                                    .field(
                                        EmbedFieldBuilder::new(
                                            "Message rate:",
                                            format!(
                                                "`${:.2} / sms`",
                                                &info.cost * self.ctx.config.price_multiplier
                                            ),
                                        )
                                        .inline(),
                                    )
                                    .validate()?
                                    .build();

                                let _ = self.ctx.bot.http.create_message(self.ctx.config.log_channel).embeds(&vec![log_embed]);

                                let number_embed = EmbedBuilder::new()
                                    .title("Success")
                                    .color(self.ctx.config.success_color)
                                    .description(format!(
                                            "You will only be charged once a message has been received.```py\n+{} {}\n```",
                                            &info.area_code, &info.phonenumber))
                                    .field(EmbedFieldBuilder::new("Service:", 
                                            &info.service).inline())
                                    .field(EmbedFieldBuilder::new("Country:", format!("{}  :flag_{}:", 
                                                &info.country, &country_price_info.iso.to_lowercase())).inline())
                                    .field(EmbedFieldBuilder::new("Message rate:", format!("`${:.2} / sms`",
                                                &info.cost * self.ctx.config.price_multiplier)).inline())
                                    .field(EmbedFieldBuilder::new("Number:", &info.number.to_string()).inline())
                                    .field(EmbedFieldBuilder::new("Expires:", format!("<t:{}:R>", &info.expiration)).inline())
                                    .field(EmbedFieldBuilder::new("Balance:", format!("`${:.2} USD`", 
                                                (user_data.balance as f32) / 100.00)).inline()) 
                                    .validate()?
                                    .build();

                                self.handle
                                    .reply(Reply::new().embed(number_embed).ephemeral())
                                    .await?;
                            }
                        }
                    }
                }
            }
        }
        Ok(())
    }
}
