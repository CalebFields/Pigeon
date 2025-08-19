use eframe::egui;
 
 

#[derive(Clone, Copy, PartialEq, Eq)]
enum Tab {
    Inbox,
    Compose,
    Contacts,
    MyAddress,
}

enum Mode {
    Onboarding,
    Main,
}

struct App {
    core: secure_p2p_msg::api::Core,
    inbox: Vec<(uuid::Uuid, Vec<u8>)>,
    mode: Mode,
    passphrase: String,
    status: String,
    active: Tab,
    // Inbox/search
    search: String,
    // Compose
    compose_contact: String,
    compose_body: String,
    compose_high: bool,
    // Contacts
    contacts: Vec<secure_p2p_msg::storage::contacts::Contact>,
    new_contact_name: String,
    new_contact_addr: String,
    new_contact_pubhex: String,
    // My Address (computed on load)
    my_addr: String,
    my_id: String,
}

impl Default for App {
    fn default() -> Self {
        let core = secure_p2p_msg::api::Core::default();
        let mode = match core.get_app_state() {
            Ok(s) if s.onboarded => Mode::Main,
            _ => Mode::Onboarding,
        };
        let inbox = if let Mode::Main = mode {
            core.inbox_list().unwrap_or_default()
        } else {
            Vec::new()
        };
        let contacts = core.contacts_list().unwrap_or_default();
        // Precompute My Address and ID before moving `core`
        #[cfg(feature = "network")]
        let (my_addr, my_id) = {
            let preview = core.ensure_identity_and_preview().ok();
            let peer = preview
                .as_ref()
                .map(|p| p.libp2p_peer_id.clone())
                .unwrap_or_default();
            let net = core.get_network_settings();
            let listen = net
                .listen_addr
                .unwrap_or_else(|| "/ip4/0.0.0.0/tcp/0".to_string());
            let addr = if peer.is_empty() {
                listen.clone()
            } else {
                format!("{}/p2p/{}", listen, peer)
            };
            (addr, if peer.is_empty() { "".to_string() } else { peer })
        };
        #[cfg(not(feature = "network"))]
        let (my_addr, my_id) = (
            "network feature disabled".to_string(),
            "network feature disabled".to_string(),
        );
        Self {
            core,
            inbox,
            mode,
            passphrase: String::new(),
            status: String::new(),
            active: Tab::Inbox,
            search: String::new(),
            compose_contact: String::new(),
            compose_body: String::new(),
            compose_high: false,
            contacts,
            new_contact_name: String::new(),
            new_contact_addr: String::new(),
            new_contact_pubhex: String::new(),
            my_addr,
            my_id,
        }
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _: &mut eframe::Frame) {
        // Poll inbox watcher for updates and raise a system notification for latest
        // (Use a lightweight single-thread runtime for inbox watcher events)
        // In this minimal example, we skip if no watcher is present.
        match self.mode {
            Mode::Onboarding => {
                egui::CentralPanel::default().show(ctx, |ui| {
                    ui.heading("Welcome to Pigeon");
                    ui.label("Set an optional passphrase to protect local data; you can also proceed without one.");
                    ui.horizontal(|ui| {
                        ui.label("Passphrase (optional):");
                        ui.text_edit_singleline(&mut self.passphrase);
                    });
                    if ui.button("Generate Identity").clicked() {
                        match self.core.ensure_identity_and_preview() {
                            Ok(preview) => {
                                if !self.passphrase.is_empty() {
                                    if let Err(e) = self.core.set_passphrase(&self.passphrase) {
                                        self.status = format!("Failed to set passphrase: {e}");
                                        return;
                                    }
                                }
                                self.status = format!("Identity created. Box PK: {}", &preview.sodium_box_pk_hex[..8.min(preview.sodium_box_pk_hex.len())]);
                            }
                            Err(e) => self.status = format!("Error: {e}"),
                        }
                    }
                    if ui.button("Enter App").clicked() {
                        let _ = self.core.set_app_state(secure_p2p_msg::settings::AppState { onboarded: true });
                        self.mode = Mode::Main;
                        self.inbox = self.core.inbox_list().unwrap_or_default();
                    }
                    if !self.status.is_empty() {
                        ui.separator();
                        ui.label(&self.status);
                    }
                });
            }
            Mode::Main => {
                egui::TopBottomPanel::top("top").show(ctx, |ui| {
                    ui.horizontal(|ui| {
                        ui.selectable_value(&mut self.active, Tab::Inbox, "Inbox");
                        ui.selectable_value(&mut self.active, Tab::Compose, "Compose");
                        ui.selectable_value(&mut self.active, Tab::Contacts, "Contacts");
                        ui.selectable_value(&mut self.active, Tab::MyAddress, "My Address");
                    });
                });
                egui::CentralPanel::default().show(ctx, |ui| match self.active {
                    Tab::Inbox => {
                        ui.horizontal(|ui| {
                            ui.label("Search:");
                            let changed = ui.text_edit_singleline(&mut self.search).changed();
                            if ui.button("Refresh").clicked() || (changed && self.search.is_empty()) {
                                self.inbox = self.core.inbox_list().unwrap_or_default();
                            }
                            if changed && !self.search.is_empty() {
                                self.inbox = self
                                    .core
                                    .inbox_search(&self.search, None)
                                    .unwrap_or_default();
                            }
                        });
                        ui.separator();
                        egui::ScrollArea::vertical().show(ui, |ui| {
                            for (id, bytes) in &self.inbox {
                                let preview = String::from_utf8_lossy(bytes);
                                ui.label(format!("{}: {}", id, preview));
                                ui.separator();
                            }
                        });
                    }
                    Tab::Compose => {
                        ui.heading("Compose Message");
                        ui.horizontal(|ui| {
                            ui.label("To (contact name or id):");
                            ui.text_edit_singleline(&mut self.compose_contact);
                        });
                        ui.text_edit_multiline(&mut self.compose_body);
                        ui.checkbox(&mut self.compose_high, "High priority");
                        if ui.button("Send").clicked() {
                            let mut send_res = Err("no contact".to_string());
                            // Resolve contact by name or id
                            let mut recipient_id: Option<u64> = None;
                            let mut recipient_pk_hex: Option<String> = None;
                            if let Ok(id) = self.compose_contact.parse::<u64>() {
                                if let Ok(Some(c)) = self.core.contacts_get(id) {
                                    recipient_id = Some(c.id);
                                    recipient_pk_hex = Some(hex::encode(c.public_key));
                                }
                            } else if let Ok(Some(c)) = self.core.contacts_find_by_name(&self.compose_contact) {
                                recipient_id = Some(c.id);
                                recipient_pk_hex = Some(hex::encode(c.public_key));
                            }
                            if let (Some(cid), Some(pkhex)) = (recipient_id, recipient_pk_hex) {
                                let body = self.compose_body.clone();
                                let high = self.compose_high;
                                let res = futures::executor::block_on(
                                    self.core.send_encrypt_and_enqueue(&pkhex, cid, &body, high),
                                );
                                if res.is_ok() {
                                    self.status = "Enqueued".to_string();
                                    self.compose_body.clear();
                                } else {
                                    self.status = "Send failed".to_string();
                                }
                                let _dummy: Result<(), String> = Ok(());
                                send_res = _dummy;
                            }
                            if send_res.is_err() {
                                self.status = "Contact not found".to_string();
                            }
                        }
                        if !self.status.is_empty() { ui.label(&self.status); }
                    }
                    Tab::Contacts => {
                        ui.horizontal(|ui| {
                            if ui.button("Refresh").clicked() {
                                self.contacts = self.core.contacts_list().unwrap_or_default();
                            }
                        });
                        ui.separator();
                        egui::ScrollArea::vertical().max_height(200.0).show(ui, |ui| {
                            for c in &self.contacts {
                                ui.horizontal(|ui| {
                                    ui.label(format!("{}: {}", c.id, c.name));
                                    if ui.button("Remove").clicked() {
                                        let _ = self.core.contacts_remove(c.id);
                                    }
                                });
                            }
                        });
                        ui.separator();
                        ui.heading("Add Contact");
                        ui.horizontal(|ui| {
                            ui.label("Name:");
                            ui.text_edit_singleline(&mut self.new_contact_name);
                        });
                        ui.horizontal(|ui| {
                            ui.label("Multiaddr:");
                            ui.text_edit_singleline(&mut self.new_contact_addr);
                        });
                        ui.horizontal(|ui| {
                            ui.label("PubKey (hex):");
                            ui.text_edit_singleline(&mut self.new_contact_pubhex);
                        });
                        if ui.button("Add").clicked() {
                            match self.core.contacts_add(
                                &self.new_contact_name,
                                &self.new_contact_addr,
                                &self.new_contact_pubhex,
                            ) {
                                Ok(_) => {
                                    self.status = "Contact added".to_string();
                                    self.new_contact_name.clear();
                                    self.new_contact_addr.clear();
                                    self.new_contact_pubhex.clear();
                                    self.contacts = self.core.contacts_list().unwrap_or_default();
                                }
                                Err(e) => self.status = format!("Add failed: {e}"),
                            }
                        }
                        if !self.status.is_empty() { ui.label(&self.status); }
                    }
                    Tab::MyAddress => {
                        ui.heading("Your ID and dialable address");
                        ui.horizontal(|ui| {
                            ui.label("Your ID:");
                            ui.text_edit_singleline(&mut self.my_id);
                            if ui.button("Copy").clicked() {
                                ui.output_mut(|o| o.copied_text = self.my_id.clone());
                            }
                        });
                        ui.separator();
                        ui.label("Your dialable address:");
                        ui.horizontal(|ui| {
                            ui.text_edit_singleline(&mut self.my_addr);
                            if ui.button("Copy").clicked() {
                                ui.output_mut(|o| o.copied_text = self.my_addr.clone());
                            }
                        });
                        ui.label("Share this with peers so they can connect to you.");
                    }
                });
            }
        }
    }
}

fn main() -> eframe::Result<()> {
    let native_options = eframe::NativeOptions::default();
    eframe::run_native(
        "Pigeon GUI",
        native_options,
        Box::new(|_cc| Box::<App>::default()),
    )
}


