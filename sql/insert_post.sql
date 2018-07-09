INSERT INTO POSTS
(guid, feed_id, feed_title, title, link, author, content)
VALUES
($1, $2, $3, $4, $5, $6, $7)
ON CONFLICT DO NOTHING;
