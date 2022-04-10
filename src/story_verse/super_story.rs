use crate::{SenderLink, Spark};
use crate::sub_story::SubStory;

pub trait SuperStory {
	// TODO Move registration of reports_link into SubStory.
	fn sub_story<S: Spark + Send + 'static>(&self, spark: S, reports_link: Option<SenderLink<S::Report>>) -> SubStory;
}
