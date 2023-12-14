use egui_phosphor::thin::*;

use crate::imports::*;
use super::*;

pub struct WalletMenu { }

impl WalletMenu {
    pub fn new() -> Self {
        Self { }
    }

    pub fn render(&mut self, core: &mut Core, ui : &mut Ui, max_height: f32) {

        let (wallet_name,wallet_filename) = if let Some(wallet_descriptor) = core.wallet_descriptor.as_ref() {
            (wallet_descriptor.title.as_deref().unwrap_or(wallet_descriptor.filename.as_str()).to_string(),wallet_descriptor.filename.clone())
        } else {
            ui.label("Missing wallet descriptor");
            return;
        };

        PopupPanel::new(ui, "wallet_selector_popup",|ui|{ ui.add(Label::new(format!("{} {} ⏷", i18n("Wallet:"), wallet_name)).sense(Sense::click())) }, |ui, _| {

            ScrollArea::vertical()
                .id_source("wallet_selector_popup_scroll")
                .auto_shrink([true; 2])
                .show(ui, |ui| {

                    let mut wallet_list = core.wallet_list().clone();
                    wallet_list.sort();
                    wallet_list.into_iter().for_each(|wallet_descriptor| {

                        let title = if let Some(title) = wallet_descriptor.title.clone() {
                            title
                        } else if wallet_descriptor.filename.as_str() == "kaspa" {
                            "Kaspa Wallet".to_string()
                        } else {
                            "NO NAME".to_string()
                        };


                        let icon = if wallet_descriptor.filename == wallet_filename {
                            // Composite::Icon(egui_phosphor::thin::LOCK_KEY_OPEN)
                            // Composite::Icon(egui_phosphor::thin::COINS)

                            if !core.state().is_connected() {
                                RichText::new(egui_phosphor::thin::CLOUD_X)
                            } else if !core.state().is_synced() {
                                RichText::new(egui_phosphor::thin::CLOUD_ARROW_DOWN)
                            } else {
                                RichText::new(egui_phosphor::thin::CLOUD_CHECK)
                            }

                        } else {
                            RichText::new(egui_phosphor::thin::FINGERPRINT_SIMPLE).color(Color32::DARK_GRAY)
                        };

                        if ui.add(CompositeButton::image_and_text(
                            Composite::icon(icon),
                            title,
                            wallet_descriptor.filename.clone(),
                        )).clicked()
                        {
                            core.get_mut::<modules::WalletOpen>().open(wallet_descriptor.clone());
                            core.select::<modules::WalletOpen>();
                        }
                    });

                    ui.label("");
                    ui.separator();
                    ui.label("");

                    if ui.medium_button(
                        "Create New Wallet",
                    ).clicked()
                    {
                        core.select::<modules::WalletCreate>();
                    }
                });

        })
        .with_min_width(240.)
        .with_max_height(max_height)
        .with_close_on_interaction(true)
        .build(ui);

    }
}

pub struct AccountMenu { }

impl AccountMenu {
    pub fn new() -> Self {
        Self { }
    }

    pub fn render(&mut self, core: &mut Core, ui : &mut Ui, account_manager : &mut AccountManager, rc : &RenderContext<'_>, max_height: f32) {
        let RenderContext { account, network_type, .. } = rc;

        PopupPanel::new(ui, "account_selector_popup",|ui|{ ui.add(Label::new(format!("{} {} ⏷",i18n("Account:"), account.name_or_id())).sense(Sense::click())) }, |ui, close| {

            egui::ScrollArea::vertical()
                .id_source("account_selector_popup_scroll")
                .auto_shrink([true; 2])
                .show(ui, |ui| {

                    let mut account_list = if let Some(account_collection) = core.account_collection() {
                        account_collection.list().clone()
                    } else {
                        ui.label("No accounts found");
                        return;
                    };

                    if let Some(prv_key_data_map) = core.prv_key_data_map() {
                        
                        for prv_key_data_info in prv_key_data_map.values() {
                            CollapsingHeader::new(prv_key_data_info.name_or_id())
                                // .default_open(true)
                                .open(Some(true))
                                .show(ui, |ui| {

                                    account_list.retain(|selectable_account|{
                                        if selectable_account.descriptor().prv_key_data_id() == Some(&prv_key_data_info.id) {

                                            if ui.account_selector_button(selectable_account, network_type, account.id() == selectable_account.id()).clicked() {
                                                account_manager.request_estimate();
                                                account_manager.state = AccountManagerState::Overview {
                                                    account: selectable_account.clone(),
                                                };
                                            }

                                            false
                                        } else {
                                            true
                                        }
                                    });
                            });
                        }
                    }

                    if account_list.is_not_empty() {
                        
                        ui.separator();

                        account_list.iter().for_each(|selectable_account|{
                            if ui.account_selector_button(selectable_account, network_type, account.id() == selectable_account.id()).clicked() {
                                account_manager.request_estimate();
                                account_manager.state = AccountManagerState::Overview {
                                    account: selectable_account.clone(),
                                };
                            }
                        });
                    }

                    ui.add_space(8.);
                    ui.separator();
                    ui.add_space(8.);
                    if ui.add(CompositeButton::opt_image_and_text(
                        Some(Composite::icon(LIST)),
                        // Composite::Icon(egui_phosphor::thin::LOCK_KEY_OPEN),
                        Some("Create New Account".into()),
                        None,
                    )).clicked() {
                        *close = true;
                        core.select::<modules::AccountCreate>();
                    }

                });

        })
        .with_min_width(240.)
        .with_max_height(max_height)
        // .with_caption("Accounts")
        // .with_close_button(true)    
        .with_close_on_interaction(true)
        .build(ui);
    }
}


pub struct ToolsMenu { }

impl ToolsMenu {
    pub fn new() -> Self {
        Self { }
    }
    pub fn render(&mut self, _core: &mut Core, ui : &mut Ui, _account_manager : &mut AccountManager, _rc : &RenderContext<'_>, max_height: f32) {

        PopupPanel::new(ui, "tools_popup",|ui|{ ui.add(Label::new(i18n("Tools ⏷")).sense(Sense::click())) }, |ui, _| {

            egui::ScrollArea::vertical()
                .id_source("tools_popup_scroll")
                .auto_shrink([true; 2])
                .show(ui, |ui| {

                    let _ = ui.button("Create Account");
                    let _ = ui.button("Import");
                    let _ = ui.button("Export");
                    // ui.button("Export");

                    // if let Some(account_collection) = core.account_collection() {
                    //     account_collection.iter().for_each(|account| {
                    //         if ui
                    //             .button(format!("Select {}\n{}", account.name_or_id(),account.balance().map(|balance|sompi_to_kaspa_string_with_suffix(balance.mature, network_type)).unwrap_or("N/A".to_string())))
                    //             .clicked()
                    //         {
                    //             account_manager.state = AccountManagerState::Overview {
                    //                 account: account.clone(),
                    //             };
                    //         }
                    //     });

                    //     ui.label("");
                    //     ui.separator();
                    //     ui.label("");
                    //     use egui_phosphor::light::FOLDER_NOTCH_PLUS;
                    //     if ui.medium_button(format!("{FOLDER_NOTCH_PLUS} Create New Account")).clicked() {
                    //         core.select::<modules::AccountCreate>();
                    //     }
                    // }

                });

        })
        .with_min_width(240.)
        .with_max_height(max_height)
        .with_close_on_interaction(true)
        .build(ui);
    }
}

