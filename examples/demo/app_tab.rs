use yui::{ArcYard, Before, Pack, SenderLink, yard};
use yui::palette::{FillColor, FillGrade};
use yui::yard::{TabBar, TabItem};

use crate::app_bar;

const BASE_ID: i32 = 50000;
const DIALOG_ID: i32 = BASE_ID + 1;
const FORM_LIST_ID: i32 = BASE_ID + 2;
const SELECTOR_LIST_ID: i32 = BASE_ID + 3;
const TEXT_ID: i32 = BASE_ID + 4;
const BUTTONS_ID: i32 = BASE_ID + 5;

#[derive(Debug, Copy, Clone)]
pub enum AppTab {
	Dialog,
	FormList,
	SelectorList,
	Text,
	Buttons,
}

impl AppTab {
	pub fn from_index(index: usize) -> Self {
		let (id, _) = TABBAR[index];
		match id {
			DIALOG_ID => AppTab::Dialog,
			FORM_LIST_ID => AppTab::FormList,
			SELECTOR_LIST_ID => AppTab::SelectorList,
			TEXT_ID => AppTab::Text,
			BUTTONS_ID => AppTab::Buttons,
			_ => unimplemented!("No tab for index {}", index)
		}
	}

	pub fn id(&self) -> i32 {
		match self {
			AppTab::Dialog => DIALOG_ID,
			AppTab::FormList => FORM_LIST_ID,
			AppTab::SelectorList => SELECTOR_LIST_ID,
			AppTab::Text => TEXT_ID,
			AppTab::Buttons => BUTTONS_ID,
		}
	}

	pub fn index(&self) -> usize {
		let self_id = self.id();
		let index = TABBAR.iter()
			.enumerate()
			.map(|(index, (id, _))| (index, *id))
			.find(|(_, id)| *id == self_id)
			.map(|(index, _)| index).expect("AppTab in TABS");
		index
	}

	pub fn page(&self, content: ArcYard, _select_tab: Option<SenderLink<usize>>) -> ArcYard {
		content
	}

	pub fn main_page(content: ArcYard, index: usize, select_tab: Option<SenderLink<usize>>) -> ArcYard {
		tab_page(content, index, select_tab)
	}
}

fn tab_page(content: ArcYard, active_tab_index: usize, select_tab: Option<SenderLink<usize>>) -> ArcYard {
	let select_tab = match select_tab {
		None => SenderLink::ignore(),
		Some(link) => link.clone(),
	};
	let tab_bar = TabBar::new(
		TABBAR.iter().map(|(id, label)| TabItem::new(*id, *label)).collect::<Vec<_>>(),
		active_tab_index,
		select_tab,
	);
	let tabbar = yard::tabbar(&tab_bar);
	content
		.pack_top(3, tabbar)
		.pack_top(3, app_bar())
		.before(yard::fill(FillColor::Background, FillGrade::Plain))
}

static TABBAR: &'static [(i32, &str)] = &[
	(TEXT_ID, "Text"),
	(BUTTONS_ID, "Buttons"),
	(DIALOG_ID, "Dialog"),
	(FORM_LIST_ID, "Form List"),
	(SELECTOR_LIST_ID, "Selector List"),
];
