use twilight_interactions::command::{CommandModel, CreateCommand};
use sparkle_convenience::reply::Reply;
use twilight_util::builder::embed::{EmbedBuilder, EmbedFieldBuilder};

use super::InteractionContext;

#[derive(CommandModel, CreateCommand)]
#[command(name = "get_countries", desc = "Lists all the supported countries")]
pub struct Command;

impl InteractionContext<'_> {
    pub async fn handle_get_countries_command(self) -> Result<(), anyhow::Error> {
        let countries_response = self.ctx.sms.clone().get_countries_list().await;
        match countries_response {
            Some(countries) => {
                let mut embed = EmbedBuilder::new().title("Countries").description("List of all supported countries");
                let mut countries_added = 0;
                for country in &countries {
                    if countries_added > 23 {
                        break;
                    }
                    let field = EmbedFieldBuilder::new(format!(":flag_{}:  {}", country.1.iso.keys().next().unwrap(), country.1.text_en.as_str()), "\u{200b}").inline();
                    embed = embed.field(field);
                    countries_added += 1;
                }
                self.handle.reply(Reply::new().content("Helped").embed(embed.validate()?.build())).await?;
            }
            None => {
                let embed = EmbedBuilder::new().title("Error").description("Error getting countries").color(0xff0000).validate()?.build();
                self.handle.reply(Reply::new().embed(embed).ephemeral()).await?;
                return Err(anyhow::anyhow!("Error"));
            }
        }
        Ok(())
    }
}