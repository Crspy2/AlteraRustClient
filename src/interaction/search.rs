use anyhow::{anyhow, Context};
use sparkle_convenience::reply::Reply;
use std::mem;
use twilight_interactions::command::{CommandModel, CreateCommand};
use twilight_model::application::interaction::InteractionData;
use twilight_util::builder::embed::{EmbedBuilder, EmbedFieldBuilder};

use super::InteractionContext;
use crate::logic::{find_similar_countries, find_similar_services, is_service_blacklisted};
use crate::sms::get_country_prices::CountryPriceInfo;

#[derive(CommandModel, CreateCommand, Debug)]
#[command(name = "search", desc = "Search our database for information")]
pub enum SearchCommand {
    #[command(name = "services")]
    Services(ServicesCommand),
    #[command(name = "prices")]
    Prices(PricesCommand),
}

impl InteractionContext<'_> {
    pub async fn handle_search_command(mut self) -> Result<(), anyhow::Error> {
        let data = match mem::take(&mut self.interaction.data) {
            Some(InteractionData::ApplicationCommand(data)) => *data,
            _ => {
                tracing::warn!("ignoring non-command interaction");
                return Err(anyhow!("Unable to get slash command info"));
            }
        };

        let command =
            SearchCommand::from_interaction(data.into()).context("Failed to parse command data");

        match command {
            Ok(SearchCommand::Services(command)) => command.execute(&self).await?,
            Ok(SearchCommand::Prices(command)) => command.execute(&self).await?,
            Err(_) => tracing::error!("Error matching search command"),
        }

        Ok(())
    }
}

#[derive(CommandModel, CreateCommand, Debug)]
#[command(name = "services", desc = "search the suppored services for a service")]
pub struct ServicesCommand {
    #[command(desc = "the service to search for", min_length = 2)]
    service: String,
}

impl ServicesCommand {
    pub async fn execute(self, ictx: &InteractionContext<'_>) -> Result<(), anyhow::Error> {
        let service: String = self.service.try_into()?;

        if is_service_blacklisted(service.as_str()) {
            let blacklisted_service_embed = EmbedBuilder::new()
                .title("Error")
                .color(ictx.ctx.config.error_color)
                .description(format!(
                    "No services similar to **{}** could be found.",
                    &service
                ))
                .validate()?
                .build();

            ictx.handle
                .reply(Reply::new().embed(blacklisted_service_embed).ephemeral())
                .await?;

            return Err(anyhow!("Blacklisted service"));
        }

        let sms_services = ictx.ctx.sms.clone().get_service_list().await;

        match sms_services {
            Err(error) => {
                let request_error_embed = EmbedBuilder::new()
                    .title("Error")
                    .color(ictx.ctx.config.error_color)
                    .description("An error occured when making your request.")
                    .validate()?
                    .build();

                ictx.handle
                    .reply(Reply::new().embed(request_error_embed).ephemeral())
                    .await?;

                return Err(anyhow!(error.errors.into_iter().nth(0).unwrap().message));
            }
            Ok(services) => {
                let similar_services = find_similar_services(&service, &services);

                if similar_services.is_empty() {
                    let no_similar_embed = EmbedBuilder::new()
                        .title("Error")
                        .color(ictx.ctx.config.error_color)
                        .description(format!(
                            "No services similar to **{}** could be found.",
                            &service
                        ))
                        .validate()?
                        .build();

                    ictx.handle
                        .reply(Reply::new().embed(no_similar_embed).ephemeral())
                        .await?;
                    return Err(anyhow!("No similar services"));
                }

                let mut embed_desc = format!(
                    "Out of `{}` services, `{}` services matched your input of **{}**:\n\n",
                    services.len(),
                    similar_services.len(),
                    service,
                );

                for (i, s) in similar_services.iter().enumerate() {
                    embed_desc += format!(
                        "**{}:** {} | `{}%`\n",
                        i + 1,
                        s.service_info.name,
                        s.similarity_score
                    )
                    .as_str();
                }

                let services_embed = EmbedBuilder::new()
                    .title("Success")
                    .color(ictx.ctx.config.success_color)
                    .description(embed_desc)
                    .validate()?
                    .build();

                ictx.handle
                    .reply(Reply::new().embed(services_embed))
                    .await?;

                return Ok(());
            }
        };
    }
}

#[derive(CommandModel, CreateCommand, Debug)]
#[command(name = "prices", desc = "Search for the price of a number")]
pub struct PricesCommand {
    #[command(desc = "the service to get the prices of", min_length = 2)]
    service: String,
    #[command(
        desc = "if you only want the price for one country, specify it here",
        min_length = 2
    )]
    country: Option<String>,
}

impl PricesCommand {
    pub async fn execute(self, ictx: &InteractionContext<'_>) -> Result<(), anyhow::Error> {
        let service: String = self.service.try_into().unwrap();
        let optional_country: Option<String> = self.country.try_into().unwrap();

        let mut country_prices = ictx
            .ctx
            .sms
            .clone()
            .get_country_prices(service.as_str())
            .await
            .unwrap_or([].to_vec());

        country_prices.sort_by(|a, b| a.low_price.partial_cmp(&b.price).unwrap());

        // If service is invalid and request for prices fails
        if country_prices.is_empty() {
            let sms_services = ictx.ctx.sms.clone().get_service_list().await.unwrap();
            if sms_services.is_empty() {
                let request_error_embed = EmbedBuilder::new()
                    .title("Error")
                    .color(ictx.ctx.config.error_color)
                    .description("An error occured when making your request.")
                    .validate()?
                    .build();

                ictx.handle
                    .reply(Reply::new().embed(request_error_embed).ephemeral())
                    .await?;

                return Err(anyhow!("Empty array response"));
            }

            let similar_services = find_similar_services(service.as_str(), &sms_services);

            if similar_services.is_empty() {
                let no_services_embed = EmbedBuilder::new()
                    .title("Error")
                    .color(ictx.ctx.config.error_color)
                    .description(format!(
                        "No services similar to **{}** could be found.",
                        service
                    ))
                    .validate()?
                    .build();

                ictx.handle
                    .reply(Reply::new().embed(no_services_embed).ephemeral())
                    .await?;
                return Err(anyhow!("no similar services"));
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
                .color(ictx.ctx.config.error_color)
                .description(embed_desc)
                .validate()?
                .build();
            ictx.handle
                .reply(Reply::new().embed(invalid_service_error).ephemeral())
                .await?;

            return Ok(());
        }

        match optional_country {
            Some(country) => {
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
                        println!("No");
                        let supported_countries = country_prices.clone();
                        let similar_countries =
                            find_similar_countries(country.as_str(), &supported_countries);

                        if similar_countries.is_empty() {
                            let no_countries_embed = EmbedBuilder::new()
                                .title("Error")
                                .color(ictx.ctx.config.error_color)
                                .description(format!(
                                    "No countries similar to **{}** could be found",
                                    country
                                ))
                                .validate()?
                                .build();

                            ictx.handle
                                .reply(Reply::new().embed(no_countries_embed).ephemeral())
                                .await?;
                            return Err(anyhow!("No similar countries"));
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
                                .color(ictx.ctx.config.success_color)
                                .description(embed_desc)
                                .validate()?
                                .build();

                            ictx.handle
                                .reply(Reply::new().embed(similar_countries_embed))
                                .await?;

                            return Ok(());
                        }
                    }
                    Some(country_price) => {
                        let price_embed = EmbedBuilder::new()
                            .title("Success")
                            .color(ictx.ctx.config.success_color)
                            .description(format!(
                            "The price for a number from **{}** for the service **{}** can range from `${:.2}` - `${:.2}`",
                            country_price.name,
                            service,
                            country_price.low_price * ictx.ctx.config.price_multiplier,
                            country_price.price * ictx.ctx.config.price_multiplier,
                        ))
                            .validate()?
                            .build();

                        ictx.handle.reply(Reply::new().embed(price_embed)).await?;
                        return Ok(());
                    }
                }
            }
            None => {
                let mut price_embed = EmbedBuilder::new()
                    .title("Success")
                    .color(ictx.ctx.config.success_color)
                    .description(
                        format!("Here are the `{}` cheapest countries that are supported by the **{}** service",
                            if country_prices.len() >= 25 { 25 } else { country_prices.len() }, 
                            service
                        )
                    );

                for info in country_prices.iter().take(25) {
                    // price_embed = price_embed.clone().field(
                    //     EmbedFieldBuilder::new(format!("{}  :flag_{}:", info.name, info.iso.to_lowercase()), format!("`${:.2}`", info.low_price * ictx.ctx.config.price_multiplier));
                    price_embed = price_embed.clone().field(EmbedFieldBuilder::new(format!("{}  :flag_{}:", info.name, info.iso.to_lowercase()), format!("`${:.2}` | `{}%` success rate", info.low_price * ictx.ctx.config.price_multiplier, info.success_rate )).inline());
                }

                ictx.handle
                    .reply(
                        Reply::new()
                            .embed(price_embed.validate()?.build()),
                    )
                    .await?;
                return Ok(());
            }
        }
    }
}
