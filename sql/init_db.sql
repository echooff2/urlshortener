CREATE USER url_shortener WITH PASSWORD 'password';

CREATE OR REPLACE FUNCTION public.random_name(IN length integer)
    RETURNS character varying
    LANGUAGE 'plpgsql'
    VOLATILE
    PARALLEL UNSAFE
    COST 100
    
AS $BODY$
DECLARE
	a varchar[] := ARRAY[]::varchar[];
	b varchar := '';
BEGIN
	FOR n IN 1..length
	LOOP
		b := chr(cast(floor(random() * 26) + 97 as int));
		a := a || b;
	END LOOP;
	
	RETURN array_to_string(a, '');
END;

$BODY$;

CREATE OR REPLACE FUNCTION public.random_unique_name(IN length integer)
    RETURNS character varying
    LANGUAGE 'plpgsql'
    VOLATILE
    PARALLEL UNSAFE
    COST 100
    
AS $BODY$
DECLARE
	namee varchar := '';
	tries int := 0;
BEGIN
	LOOP
		IF tries >= 100 THEN
			RAISE EXCEPTION SQLSTATE '90001' USING MESSAGE = 'couldn''t generate unique name within 100 tries';
			RETURN null;
		END IF;
		namee := random_name(length);
		IF NOT EXISTS (SELECT FROM redirect WHERE "name" = namee) THEN
			RETURN namee;
		END IF;
		tries := tries + 1;
	END LOOP;
END;
$BODY$;

CREATE SEQUENCE public.redirect_id_seq
    AS integer
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;

CREATE TABLE public.redirect (
    id integer DEFAULT nextval('public.redirect_id_seq'::regclass) NOT NULL,
    name character varying(3) DEFAULT public.random_unique_name(3) NOT NULL,
    url character varying NOT NULL
);

CREATE TABLE public."user" (
    id integer NOT NULL,
    password character varying NOT NULL
);

CREATE SEQUENCE public.user_id_seq
    AS integer
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;

ALTER TABLE ONLY public."user" ALTER COLUMN id SET DEFAULT nextval('public.user_id_seq'::regclass);

ALTER TABLE ONLY public.redirect
    ADD CONSTRAINT redirect_pkey PRIMARY KEY (id);

ALTER TABLE ONLY public.redirect
ADD CONSTRAINT redirect_unique_name UNIQUE (name);

ALTER TABLE ONLY public."user"
    ADD CONSTRAINT user_pkey PRIMARY KEY (id);

GRANT ALL ON SEQUENCE public.redirect_id_seq TO url_shortener;

GRANT SELECT,INSERT ON TABLE public.redirect TO url_shortener;

GRANT SELECT ON TABLE public."user" TO url_shortener;