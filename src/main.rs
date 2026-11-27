use std::{
    env, fs,
    path::{Path, PathBuf},
    time::Duration,
};

use eframe::egui;
use serde::{Deserialize, Serialize};
use tray_icon::{
    Icon, TrayIcon, TrayIconBuilder,
    menu::{Menu, MenuEvent, MenuId, MenuItem, PredefinedMenuItem},
};

#[derive(Debug, Deserialize, Serialize)]
struct Track {
    title: String,
    artist: String,
    cid: String,
}

impl Track {
    fn new(title: &str, artist: &str, cid: &str) -> Result<Self, &'static str> {
        let title = title.trim();
        let artist = artist.trim();
        let cid = cid.trim();

        if title.is_empty() || artist.is_empty() || cid.is_empty() {
            return Err("Enter a title, creator, and IPFS CID.");
        }

        Ok(Self {
            title: title.to_owned(),
            artist: artist.to_owned(),
            cid: cid.to_owned(),
        })
    }
}

struct ChoiresyApp {
    title: String,
    artist: String,
    cid: String,
    tracks: Vec<Track>,
    library_path: Option<PathBuf>,
    status: Option<String>,
    show_id: MenuId,
    quit_id: MenuId,
    _tray_icon: TrayIcon,
    quitting: bool,
}

impl ChoiresyApp {
    fn new(tray_icon: TrayIcon, show_id: MenuId, quit_id: MenuId) -> Self {
        let library_path = library_path();
        let (tracks, status) = match library_path.as_deref() {
            Some(path) => match load_library(path) {
                Ok(tracks) => (tracks, None),
                Err(error) => (Vec::new(), Some(error)),
            },
            None => (
                Vec::new(),
                Some("Could not find your home directory; tracks will not be saved.".into()),
            ),
        };

        Self {
            title: String::new(),
            artist: String::new(),
            cid: String::new(),
            tracks,
            library_path,
            status,
            show_id,
            quit_id,
            _tray_icon: tray_icon,
            quitting: false,
        }
    }

    fn handle_tray_events(&mut self, ctx: &egui::Context) {
        while let Ok(event) = MenuEvent::receiver().try_recv() {
            if event.id == self.show_id {
                ctx.send_viewport_cmd(egui::ViewportCommand::Visible(true));
                ctx.send_viewport_cmd(egui::ViewportCommand::Focus);
            } else if event.id == self.quit_id {
                self.quitting = true;
                ctx.send_viewport_cmd(egui::ViewportCommand::Close);
            }
        }

        if ctx.input(|input| input.viewport().close_requested()) && !self.quitting {
            ctx.send_viewport_cmd(egui::ViewportCommand::CancelClose);
            ctx.send_viewport_cmd(egui::ViewportCommand::Visible(false));
        }
    }

    fn add_track(&mut self) {
        match Track::new(&self.title, &self.artist, &self.cid) {
            Ok(track) => {
                self.tracks.push(track);
                self.title.clear();
                self.artist.clear();
                self.cid.clear();
                self.save_library();
            }
            Err(error) => self.status = Some(error.into()),
        }
    }

    fn save_library(&mut self) {
        let Some(path) = self.library_path.as_deref() else {
            return;
        };

        match save_library(path, &self.tracks) {
            Ok(()) => self.status = Some("Track added to your library.".into()),
            Err(error) => self.status = Some(error),
        }
    }
}

impl eframe::App for ChoiresyApp {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        let ctx = ui.ctx().clone();
        self.handle_tray_events(&ctx);
        ctx.request_repaint_after(Duration::from_millis(250));

        egui::CentralPanel::default().show(ui, |ui| {
            ui.heading("Choiresy");
            ui.label("A community-owned home for AI-created music.");
            ui.separator();

            ui.heading("Add a track");
            ui.horizontal(|ui| {
                ui.label("Title");
                ui.text_edit_singleline(&mut self.title);
            });
            ui.horizontal(|ui| {
                ui.label("Creator");
                ui.text_edit_singleline(&mut self.artist);
            });
            ui.horizontal(|ui| {
                ui.label("IPFS CID");
                ui.text_edit_singleline(&mut self.cid);
            });
            if ui.button("Add to library").clicked() {
                self.add_track();
            }
            if let Some(status) = &self.status {
                ui.label(status);
            }

            ui.separator();
            ui.heading("Your library");
            if self.tracks.is_empty() {
                ui.label("No tracks yet. Add an AI-created track by its IPFS CID.");
            } else {
                egui::ScrollArea::vertical().show(ui, |ui| {
                    for track in &self.tracks {
                        ui.group(|ui| {
                            ui.strong(format!("{} — {}", track.artist, track.title));
                            ui.monospace(&track.cid);
                        });
                    }
                });
            }
        });
    }
}

fn library_path() -> Option<PathBuf> {
    env::var_os("HOME")
        .or_else(|| env::var_os("USERPROFILE"))
        .map(PathBuf::from)
        .map(|home| home.join(".choiresy").join("library.json"))
}

fn load_library(path: &Path) -> Result<Vec<Track>, String> {
    match fs::read(path) {
        Ok(contents) => serde_json::from_slice(&contents)
            .map_err(|error| format!("Could not read your library: {error}")),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(Vec::new()),
        Err(error) => Err(format!("Could not read your library: {error}")),
    }
}

fn save_library(path: &Path, tracks: &[Track]) -> Result<(), String> {
    let parent = path
        .parent()
        .ok_or_else(|| "Could not determine the library folder.".to_owned())?;
    fs::create_dir_all(parent).map_err(|error| format!("Could not save your library: {error}"))?;
    let contents = serde_json::to_vec_pretty(tracks)
        .map_err(|error| format!("Could not save your library: {error}"))?;
    fs::write(path, contents).map_err(|error| format!("Could not save your library: {error}"))
}

fn tray_icon() -> Icon {
    let mut rgba = Vec::with_capacity(16 * 16 * 4);
    for y in 0..16 {
        for x in 0..16 {
            let inside = (x as i32 - 7).pow(2) + (y as i32 - 7).pow(2) <= 49;
            rgba.extend_from_slice(if inside {
                &[38, 166, 154, 255]
            } else {
                &[0, 0, 0, 0]
            });
        }
    }
    Icon::from_rgba(rgba, 16, 16).expect("generated tray icon has valid dimensions")
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let menu = Menu::new();
    let show_item = MenuItem::new("Show Choiresy", true, None);
    let quit_item = MenuItem::new("Quit", true, None);
    menu.append(&show_item)?;
    menu.append(&PredefinedMenuItem::separator())?;
    menu.append(&quit_item)?;

    let show_id = show_item.id().clone();
    let quit_id = quit_item.id().clone();
    let tray_icon = TrayIconBuilder::new()
        .with_menu(Box::new(menu))
        .with_tooltip("Choiresy")
        .with_icon(tray_icon())
        .build()?;

    eframe::run_native(
        "Choiresy",
        eframe::NativeOptions {
            viewport: egui::ViewportBuilder::default()
                .with_inner_size([560.0, 480.0])
                .with_min_inner_size([420.0, 360.0]),
            ..Default::default()
        },
        Box::new(move |_| Ok(Box::new(ChoiresyApp::new(tray_icon, show_id, quit_id)))),
    )?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{Track, load_library, save_library};

    #[test]
    fn track_requires_title_creator_and_cid() {
        assert!(Track::new("", "Creator", "bafyexample").is_err());
        assert!(Track::new("Song", " ", "bafyexample").is_err());
        assert!(Track::new("Song", "Creator", "").is_err());
    }

    #[test]
    fn track_trims_metadata_fields() {
        let track = Track::new(" Song ", " Creator ", " bafyexample ").unwrap();
        assert_eq!(track.title, "Song");
        assert_eq!(track.artist, "Creator");
        assert_eq!(track.cid, "bafyexample");
    }

    #[test]
    fn library_round_trips_local_tracks() {
        let directory = std::env::temp_dir().join(format!("choiresy-test-{}", std::process::id()));
        let path = directory.join("library.json");
        let track = Track::new("Song", "Creator", "bafyexample").unwrap();

        assert!(load_library(&path).unwrap().is_empty());
        save_library(&path, &[track]).unwrap();
        let loaded = load_library(&path).unwrap();

        assert_eq!(loaded.len(), 1);
        assert_eq!(loaded[0].title, "Song");
        assert_eq!(loaded[0].artist, "Creator");
        assert_eq!(loaded[0].cid, "bafyexample");
        std::fs::remove_dir_all(directory).unwrap();
    }
}
