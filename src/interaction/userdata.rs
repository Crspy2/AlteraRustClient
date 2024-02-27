use sparkle_convenience::{
    error::IntoError, interaction::extract::InteractionDataExt, reply::Reply,
};
use twilight_interactions::command::{CommandModel, CreateCommand};

use twilight_model::guild::Permissions;
use twilight_util::builder::embed::{EmbedBuilder, EmbedFieldBuilder, ImageSource};

use crate::api::get_user_data;

use super::InteractionContext;

fn default_permissions() -> Permissions {
    return Permissions::VIEW_AUDIT_LOG;
}

#[derive(CreateCommand, CommandModel, Debug)]
#[command(
    name = "userdata",
    desc = "retreive a user's information from our database",
    dm_permission = false,
    default_permissions = "default_permissions"
)]
pub struct UserDataCommand {
    #[command(desc = "the user to fetch the information of")]
    pub user: Option<twilight_model::user::User>,
}

impl InteractionContext<'_> {
    pub async fn handle_user_data_command(self) -> Result<(), anyhow::Error> {
        let options = UserDataCommand::from_interaction(
            self.interaction.data.clone().ok()?.command().ok()?.into(),
        )?;

        let user: twilight_model::user::User;

        match options.user {
            Some(u) => {
                user = u;
            }
            None => {
                let u = self.interaction.author().ok()?;
                user = u.clone();
            }
        };

        let user_data = get_user_data(user.id).await;

        match user_data {
            Err(_) => {
                let error_embed = EmbedBuilder::new()
                    .title("Error")
                    .color(self.ctx.config.error_color)
                    .description(format!("No account could be found for **@{}**", self.interaction.author().unwrap().name))
                    .validate()?
                    .build();

                self.handle
                    .reply(Reply::new().embed(error_embed).ephemeral())
                    .await?;
            }
            Ok(data) => {
                let info_embed = EmbedBuilder::new()
                    .title("Success")
                    .color(self.ctx.config.success_color)
                    .description(format!(
                        "Here is the requested information for <@{}>",
                        user.id
                    ))
                    .thumbnail(ImageSource::url(data.image)?)
                    .field(EmbedFieldBuilder::new(
                        "Discord User Info:",
                        format!("`{}` | **@{}**", user.id, user.name),
                    ))
                    .field(EmbedFieldBuilder::new(
                        "User Balance:",
                        format!("`${:.2} USD`", (data.balance as f32) / 100.00),
                    ))
                    .field(EmbedFieldBuilder::new(
                        "Last Deposit:",
                        format!(
                            "<t:{}:f>",
                            chrono::DateTime::parse_from_rfc3339(&data.updated_at)
                                .expect("Failed to parse datetime")
                                .with_timezone(&chrono::Utc)
                                .timestamp()
                        ),
                    ))
                    .field(EmbedFieldBuilder::new(
                        "Number of Deposits",
                        format!("`{}` deposits on record", data.invoices.len()),
                    ))
                    .field(EmbedFieldBuilder::new(
                        "Role",
                        if data.role == 2 {
                            "<:moderator:1123391823885381762> **Admin**"
                        } else if data.role == 1 {
                            "<:bug:1123148520665391134> **Support**"
                        } else {
                          "<:members:1123391833997852692> **User**"
                        },
                    ))
                    .validate()?
                    .build();

                self.handle
                    .reply(Reply::new().embed(info_embed).ephemeral())
                    .await?;
            }
        }

        Ok(())
    }
}
