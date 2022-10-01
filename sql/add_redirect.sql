INSERT INTO public."redirect" ("name", "url")
VALUES (random_unique_name($1), $2)
RETURNING $table_fields;