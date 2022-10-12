INSERT INTO public."redirect" ("name", "url")
VALUES ($1, $2)
RETURNING $table_fields;