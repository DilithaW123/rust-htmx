DROP TABLE IF EXISTS cases;

CREATE TABLE cases (
    	id SERIAL PRIMARY KEY,
    	status VARCHAR(30) NOT NULL,
	message TEXT
);
