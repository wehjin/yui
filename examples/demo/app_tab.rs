use yui::{ArcYard, Pack, SenderLink, yard};

use crate::app_bar;

#[derive(Debug, Clone)]
pub enum AppTab {
	Dialog,
	FormList,
	SelectorList,
	Text,
	Buttons,
}

impl AppTab {
	pub fn from_index(index: usize) -> Self {
		match index {
			0 => AppTab::Dialog,
			1 => AppTab::FormList,
			2 => AppTab::SelectorList,
			3 => AppTab::Text,
			4 => AppTab::Buttons,
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
		let index = TABS.iter()
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
	let tabbar = yard::tabbar(TABS, active_tab_index, select_tab);
	content
		.pack_top(3, tabbar)
		.pack_top(3, app_bar())
}

static TABS: &'static [(i32, &str)] = &[
	(DIALOG_ID, "Dialog"),
	(FORM_LIST_ID, "Form List"),
	(SELECTOR_LIST_ID, "Selector List"),
	(TEXT_ID, "Text"),
	(BUTTONS_ID, "Buttons"),
];

static DIALOG_ID: i32 = 1;
static FORM_LIST_ID: i32 = 2;
static SELECTOR_LIST_ID: i32 = 3;
static TEXT_ID: i32 = 4;
static BUTTONS_ID: i32 = 5;
