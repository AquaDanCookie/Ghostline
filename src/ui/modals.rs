use egui::{Align, Color32, CornerRadius, Frame, Id, Label, Layout, Margin, RichText, Stroke, Ui};

use crate::app::{
    Action, App, CredentialsPrompt, ImportReview, ModalState, SudoIntent, SudoPrompt,
};
use crate::format;
use crate::theme::Palette;

use super::widgets::{self, Btn};

pub fn show(app: &mut App, ctx: &egui::Context, actions: &mut Vec<Action>) {
    let Some(mut modal) = app.modal.take() else {
        return;
    };
    let p = app.palette;
    let nm_ready = app.nm_ready;
    let demo = app.demo;
    let connected = app.tunnel.is_connected();
    let busy = app.is_busy();

    let dismiss = match &mut modal {
        ModalState::Import(review) => {
            import_modal(ctx, &p, review, nm_ready, connected, busy, actions)
        }
        ModalState::ImportError { path, error } => error_modal(ctx, &p, path, error),
        ModalState::Credentials(prompt) => {
            let profile_name = app
                .profile(&prompt.profile_id)
                .map(|profile| profile.name.clone())
                .unwrap_or_else(|| "this profile".to_owned());
            credentials_modal(ctx, &p, prompt, &profile_name, demo, actions)
        }
        ModalState::Sudo(prompt) => sudo_modal(ctx, &p, prompt, demo, actions),
        ModalState::Remove(prompt) => {
            let profile = app.profile(&prompt.profile_id);
            let profile_name = profile
                .map(|profile| profile.name.clone())
                .unwrap_or_else(|| "this profile".to_owned());
            let cyber = profile
                .map(|profile| profile.is_cyberghost())
                .unwrap_or(false);
            let active = app.tunnel.profile_id() == Some(prompt.profile_id.as_str());
            remove_modal(ctx, &p, &profile_name, cyber, active, actions)
        }
        ModalState::ClearLogs => clear_logs_modal(ctx, &p, app.logs.len(), actions),
    };

    app.modal = Some(modal);
    if dismiss {
        actions.push(Action::CloseModal);
    }
}

fn modal_frame(p: &Palette) -> Frame {
    Frame::new()
        .fill(p.surface)
        .stroke(Stroke::new(1.0_f32, p.line))
        .corner_radius(CornerRadius::same(14))
        .inner_margin(Margin::same(24))
}

fn show_modal(
    ctx: &egui::Context,
    p: &Palette,
    id: &str,
    content_width: f32,
    add_contents: impl FnOnce(&mut Ui) -> bool,
) -> bool {
    let response = egui::Modal::new(Id::new(id))
        .backdrop_color(Color32::from_black_alpha(150))
        .frame(modal_frame(p))
        .show(ctx, |ui| {
            ui.set_width(content_width);
            add_contents(ui)
        });
    response.inner || response.should_close()
}

fn heading(ui: &mut Ui, p: &Palette, eyebrow: &str, title: &str, subtitle: &str) {
    widgets::eyebrow(ui, p, eyebrow);
    ui.add_space(7.0);
    ui.add(
        Label::new(
            RichText::new(title)
                .font(widgets::semibold(24.0))
                .color(p.text),
        )
        .wrap(),
    );
    if !subtitle.is_empty() {
        ui.add_space(5.0);
        ui.add(
            Label::new(
                RichText::new(subtitle)
                    .font(widgets::sans(13.5))
                    .color(p.muted),
            )
            .wrap(),
        );
    }
    ui.add_space(16.0);
    widgets::hline(ui, p);
    ui.add_space(16.0);
}

struct FooterResult {
    cancel: bool,
    primary: bool,
}

fn footer(ui: &mut Ui, p: &Palette, primary: widgets::Btn<'_>) -> FooterResult {
    ui.add_space(18.0);
    widgets::hline(ui, p);
    ui.add_space(14.0);
    let mut result = FooterResult {
        cancel: false,
        primary: false,
    };
    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
        result.primary = widgets::button(ui, p, primary).clicked();
        ui.add_space(4.0);
        result.cancel = widgets::button(ui, p, Btn::ghost("Cancel")).clicked();
    });
    result
}

fn detail_row(ui: &mut Ui, p: &Palette, label: &str, value: &str, mono: bool) {
    ui.horizontal_top(|ui| {
        widgets::label(ui, label, widgets::sans(13.0), p.muted);
        ui.with_layout(Layout::right_to_left(Align::Min), |ui| {
            let font = if mono {
                widgets::mono_medium(12.0)
            } else {
                widgets::medium(13.5)
            };
            ui.add(
                Label::new(RichText::new(value).font(font).color(p.text))
                    .wrap()
                    .halign(Align::Max),
            );
        });
    });
    ui.add_space(6.0);
}

fn import_modal(
    ctx: &egui::Context,
    p: &Palette,
    review: &ImportReview,
    nm_ready: bool,
    connected: bool,
    busy: bool,
    actions: &mut Vec<Action>,
) -> bool {
    let path = review.path.to_string_lossy();
    let location = match (
        review.parsed.country.as_deref(),
        review.parsed.city.as_deref(),
    ) {
        (Some(country), Some(city)) => format!("{country}, {city}"),
        (Some(country), None) => country.to_owned(),
        _ => "Not recognised from the file name".to_owned(),
    };
    let locked = review.duplicate && (connected || busy);
    let can_import = nm_ready && !locked;

    show_modal(ctx, p, "modal-import", 512.0, |ui| {
        heading(
            ui,
            p,
            "Import",
            &review.parsed.name,
            "Review what NetworkManager will use before the connection is created.",
        );

        detail_row(ui, p, "Protocol", review.parsed.protocol.label(), false);
        detail_row(
            ui,
            p,
            "Endpoint",
            &format!(
                "{}:{}",
                review.parsed.remote_host, review.parsed.remote_port
            ),
            true,
        );
        detail_row(ui, p, "Location", &location, false);
        detail_row(
            ui,
            p,
            "Credentials",
            if review.parsed.requires_auth {
                "Username and password"
            } else {
                "Not required"
            },
            false,
        );
        detail_row(ui, p, "Source file", &format::shorten_path(&path, 58), true);

        if review.parsed.requires_auth {
            ui.add_space(4.0);
            widgets::note(
                ui,
                p,
                "Credentials needed",
                "NetworkManager will ask for a username and password when you connect. Ghostline can save the password in the system keyring.",
                p.muted,
            );
        }

        for warning in &review.parsed.warnings {
            ui.add_space(10.0);
            widgets::note(ui, p, "Profile warning", warning, p.warn);
        }

        if review.duplicate {
            ui.add_space(10.0);
            widgets::note(
                ui,
                p,
                "Already imported",
                if connected {
                    "This profile is active. Disconnect before recreating its NetworkManager connection."
                } else if busy {
                    "Wait for the current connection attempt to finish before recreating this profile."
                } else {
                    "This file is already in Locations. Continuing re-reads it and recreates its NetworkManager connection. Favorites and the saved username stay."
                },
                if locked { p.danger } else { p.warn },
            );
        }

        if !nm_ready {
            ui.add_space(10.0);
            widgets::note(
                ui,
                p,
                "NetworkManager is not available",
                "Install NetworkManager and start its service before importing a profile.",
                p.danger,
            );
        }

        let primary = if review.duplicate {
            Btn::primary("Recreate connection")
        } else {
            Btn::primary("Import profile")
        }
        .icon(widgets::Icon::Import)
        .min_width(170.0)
        .enabled(can_import);

        let outcome = footer(ui, p, primary);
        if outcome.cancel {
            return true;
        }
        let enter = ui.input(|input| input.key_pressed(egui::Key::Enter));
        if can_import && (outcome.primary || enter) {
            actions.push(Action::ConfirmImport);
        }
        false
    })
}

fn error_modal(ctx: &egui::Context, p: &Palette, path: &str, error: &str) -> bool {
    show_modal(ctx, p, "modal-import-error", 452.0, |ui| {
        heading(
            ui,
            p,
            "Import",
            "Could not read that profile",
            "The file was not changed. Nothing was imported.",
        );
        detail_row(ui, p, "File", &format::shorten_path(path, 54), true);
        widgets::note(ui, p, "Read error", error, p.danger);
        ui.add_space(18.0);
        widgets::hline(ui, p);
        ui.add_space(14.0);
        let mut close = false;
        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
            if widgets::button(ui, p, Btn::secondary("Close").min_width(100.0)).clicked() {
                close = true;
            }
        });
        close
    })
}

fn credentials_modal(
    ctx: &egui::Context,
    p: &Palette,
    prompt: &mut CredentialsPrompt,
    profile_name: &str,
    demo: bool,
    actions: &mut Vec<Action>,
) -> bool {
    show_modal(ctx, p, "modal-credentials", 472.0, |ui| {
        let username_text_id = ui.make_persistent_id(("text-input", "credentials-username"));
        let password_text_id = ui.make_persistent_id(("password-input", "credentials-password"));
        if prompt.focus_requested {
            let target = if prompt.username.trim().is_empty() {
                username_text_id
            } else {
                password_text_id
            };
            ui.memory_mut(|memory| memory.request_focus(target));
            prompt.focus_requested = false;
        }

        heading(
            ui,
            p,
            "Credentials",
            "Username and password",
            &format!("NetworkManager needs credentials for {profile_name}."),
        );

        widgets::label(ui, "Username", widgets::medium(13.0), p.text);
        ui.add_space(5.0);
        let username_response = widgets::text_input(
            ui,
            p,
            "credentials-username",
            &mut prompt.username,
            "Username",
            ui.available_width(),
        );
        if username_response.changed() {
            prompt.error = None;
        }
        ui.add_space(12.0);

        widgets::label(ui, "Password", widgets::medium(13.0), p.text);
        ui.add_space(5.0);
        let password_response = widgets::password_input(
            ui,
            p,
            "credentials-password",
            &mut prompt.password,
            "Password",
            ui.available_width(),
            &mut prompt.show_password,
        );
        if password_response.changed() {
            prompt.error = None;
        }
        ui.add_space(12.0);

        ui.horizontal_top(|ui| {
            if widgets::toggle(
                ui,
                p,
                "remember-password",
                prompt.remember,
                "Save password in the system keyring",
            )
            .clicked()
            {
                prompt.remember = !prompt.remember;
            }
            ui.add_space(4.0);
            ui.vertical(|ui| {
                widgets::label(
                    ui,
                    "Save password in the system keyring",
                    widgets::medium(13.5),
                    p.text,
                );
                ui.add_space(2.0);
                widgets::caption(
                    ui,
                    p,
                    "Uses secret-tool. Without this, the password is passed to NetworkManager once and discarded.",
                );
            });
        });

        if let Some(error) = &prompt.error {
            ui.add_space(10.0);
            widgets::note(ui, p, "Cannot continue", error, p.danger);
        }
        if demo {
            ui.add_space(10.0);
            widgets::caption(
                ui,
                p,
                "Demo mode does not touch the keyring or NetworkManager.",
            );
        }

        let outcome = footer(
            ui,
            p,
            Btn::primary("Connect")
                .icon(widgets::Icon::Power)
                .min_width(132.0),
        );
        if outcome.cancel {
            return true;
        }

        let enter = ui.input(|input| input.key_pressed(egui::Key::Enter));
        let ready = !prompt.username.trim().is_empty() && !prompt.password.is_empty();
        if outcome.primary || (enter && ready) {
            actions.push(Action::SubmitCredentials);
        }
        false
    })
}

fn sudo_modal(
    ctx: &egui::Context,
    p: &Palette,
    prompt: &mut SudoPrompt,
    demo: bool,
    actions: &mut Vec<Action>,
) -> bool {
    show_modal(ctx, p, "modal-sudo", 472.0, |ui| {
        let password_text_id = ui.make_persistent_id(("password-input", "sudo-password"));
        if prompt.focus_requested {
            ui.memory_mut(|memory| memory.request_focus(password_text_id));
            prompt.focus_requested = false;
        }

        let (title, subtitle) = match prompt.intent {
            SudoIntent::Connect(_) => (
                "Open the tunnel",
                "cyberghostvpn uses sudo to start the connection. The password is sent to sudo over stdin.",
            ),
            SudoIntent::Disconnect => (
                "Close the tunnel",
                "cyberghostvpn uses sudo to stop the connection. The password is sent to sudo over stdin.",
            ),
        };

        heading(ui, p, "Administrator", title, subtitle);

        widgets::label(ui, "Password", widgets::medium(13.0), p.text);
        ui.add_space(5.0);
        let response = widgets::password_input(
            ui,
            p,
            "sudo-password",
            &mut prompt.password,
            "Administrator password",
            ui.available_width(),
            &mut prompt.show_password,
        );
        if response.changed() {
            prompt.error = None;
        }
        ui.add_space(12.0);

        ui.horizontal_top(|ui| {
            if widgets::toggle(
                ui,
                p,
                "remember-sudo",
                prompt.remember,
                "Keep the administrator password for this session",
            )
            .clicked()
            {
                prompt.remember = !prompt.remember;
            }
            ui.add_space(4.0);
            ui.vertical(|ui| {
                widgets::label(
                    ui,
                    "Keep for this session",
                    widgets::medium(13.5),
                    p.text,
                );
                ui.add_space(2.0);
                widgets::caption(
                    ui,
                    p,
                    "Held in memory so reconnects and disconnects do not ask again. It is never written to disk.",
                );
            });
        });

        if let Some(error) = &prompt.error {
            ui.add_space(10.0);
            widgets::note(ui, p, "Cannot continue", error, p.danger);
        }
        if demo {
            ui.add_space(10.0);
            widgets::caption(ui, p, "Demo mode does not use sudo.");
        }

        let label = match prompt.intent {
            SudoIntent::Connect(_) => "Connect",
            SudoIntent::Disconnect => "Disconnect",
        };
        let outcome = footer(
            ui,
            p,
            Btn::primary(label)
                .icon(widgets::Icon::Power)
                .min_width(132.0),
        );
        if outcome.cancel {
            return true;
        }

        let enter = ui.input(|input| input.key_pressed(egui::Key::Enter));
        if outcome.primary || (enter && !prompt.password.is_empty()) {
            actions.push(Action::SubmitSudo);
        }
        false
    })
}

fn remove_modal(
    ctx: &egui::Context,
    p: &Palette,
    profile_name: &str,
    cyber: bool,
    active: bool,
    actions: &mut Vec<Action>,
) -> bool {
    show_modal(ctx, p, "modal-remove", 452.0, |ui| {
        heading(
            ui,
            p,
            "Remove",
            &format!("Remove {profile_name}?"),
            if cyber {
                "Ghostline removes the saved location. Your CyberGhost account, the official CLI, and the active tunnel stay as they are."
            } else {
                "Ghostline removes the saved profile and asks NetworkManager to delete its connection. The original profile file is not deleted."
            },
        );

        detail_row(
            ui,
            p,
            if cyber {
                "Saved location"
            } else {
                "NetworkManager connection"
            },
            &format!("Ghostline: {profile_name}"),
            true,
        );

        if active {
            ui.add_space(6.0);
            widgets::note(
                ui,
                p,
                "This profile is active",
                if cyber {
                    "Ghostline stops the tunnel through the CyberGhost CLI before removing the saved location."
                } else {
                    "Ghostline disconnects the tunnel before removing the connection."
                },
                p.warn,
            );
        }

        ui.add_space(6.0);
        widgets::caption(ui, p, "This cannot be undone.");

        let outcome = footer(
            ui,
            p,
            Btn::danger("Remove profile")
                .icon(widgets::Icon::Trash)
                .min_width(168.0),
        );
        if outcome.cancel {
            return true;
        }
        if outcome.primary {
            actions.push(Action::ConfirmRemove);
        }
        false
    })
}

fn clear_logs_modal(
    ctx: &egui::Context,
    p: &Palette,
    count: usize,
    actions: &mut Vec<Action>,
) -> bool {
    show_modal(ctx, p, "modal-clear-logs", 432.0, |ui| {
        heading(
            ui,
            p,
            "Activity",
            "Clear the activity log?",
            "This deletes the local log file and every entry currently shown.",
        );
        widgets::metric(
            ui,
            p,
            "Entries",
            &format::count(count, "entry", "entries"),
            None,
        );
        ui.add_space(8.0);
        widgets::caption(ui, p, "The log cannot be recovered after it is cleared.");
        let outcome = footer(
            ui,
            p,
            Btn::danger("Clear log")
                .icon(widgets::Icon::Trash)
                .min_width(140.0),
        );
        if outcome.cancel {
            return true;
        }
        if outcome.primary {
            actions.push(Action::ConfirmClearLogs);
        }
        false
    })
}
