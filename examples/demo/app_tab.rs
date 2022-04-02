use yui::{ArcYard, Pack, SenderLink, yard};

use crate::app_bar;

const DIALOG_ID: i32 = 1;
const FORM_LIST_ID: i32 = 2;
const SELECTOR_LIST_ID: i32 = 3;
const TEXT_ID: i32 = 4;
const BUTTONS_ID: i32 = 5;

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

	pub fn page(&self, content: ArcYard, select_tab: Option<SenderLink<usize>>) -> ArcYard {
		let index = self.index();
		tab_page(content, index, select_tab)
	}
}

fn tab_page(
	content: ArcYard,
	active_tab_index: usize,
	select_tab: Option<SenderLink<usize>>,
) -> ArcYard {
	let select_tab = match select_tab {
		None => SenderLink::ignore(),
		Some(link) => link.clone(),
	};
	let tabbar = yard::tabbar(TABBAR, active_tab_index, select_tab);
	content
		.pack_top(3, tabbar)
		.pack_top(3, app_bar())
}

static TABBAR: &'static [(i32, &str)] = &[
	(TEXT_ID, "Text"),
	(BUTTONS_ID, "Buttons"),
	(DIALOG_ID, "Dialog"),
	(FORM_LIST_ID, "Form List"),
	(SELECTOR_LIST_ID, "Selector List"),
];
