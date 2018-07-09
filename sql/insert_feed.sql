INSERT INTO feeds (url, title) values ($1, $2) RETURNING id;
