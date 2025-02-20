use crate::db::score::Score;
use crate::db::{NodeDB, IncomingPost, PostId};
use crate::db::handle_post::POSTS_TABLE;

use crate::misc::get_epoch;

pub trait Search {
    fn search_posts(&self, after: Option<PostId>, max_results:usize) -> Result<Vec<(IncomingPost, f64)>, Box<dyn std::error::Error>>;
}

impl Search for NodeDB {
    fn search_posts(&self, after: Option<PostId>, max_results:usize) -> Result<Vec<(IncomingPost, f64)>, Box<dyn std::error::Error>> {
        let posts = self.db.open_tree(POSTS_TABLE)?;

        let mut all_posts = vec![];
        let current_time = get_epoch();
        for (_idx, post) in posts.iter().enumerate() {

            if all_posts.len() >= max_results {
                continue;
            }

            if let Ok((_post_id, post)) = post {
                let post: IncomingPost = bincode::deserialize(&post)?;
                
                let author_score = self.get_score(&post.post.author, 1200)? as f64;
                let seconds_ago = (current_time - post.received) as f64;

                let post_score = author_score.log10() / seconds_ago; // reddit rank

                all_posts.push((post, post_score));
            }
        }

        Ok(all_posts)
    }

}