SELECT id, title, saved FROM posts WHERE feed_id = $1 ORDER BY id DESC;
