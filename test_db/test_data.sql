--
-- PostgreSQL database dump
--

-- Dumped from database version 15.2 (Debian 15.2-1.pgdg110+1)
-- Dumped by pg_dump version 15.2 (Debian 15.2-1.pgdg110+1)

SET statement_timeout = 0;
SET lock_timeout = 0;
SET idle_in_transaction_session_timeout = 0;
SET client_encoding = 'UTF8';
SET standard_conforming_strings = on;
SELECT pg_catalog.set_config('search_path', '', false);
SET check_function_bodies = false;
SET xmloption = content;
SET client_min_messages = warning;
SET row_security = off;

--
-- Name: diesel_manage_updated_at(regclass); Type: FUNCTION; Schema: public; Owner: postgres
--

CREATE FUNCTION public.diesel_manage_updated_at(_tbl regclass) RETURNS void
    LANGUAGE plpgsql
    AS $$
BEGIN
    EXECUTE format('CREATE TRIGGER set_updated_at BEFORE UPDATE ON %s
                    FOR EACH ROW EXECUTE PROCEDURE diesel_set_updated_at()', _tbl);
END;
$$;


ALTER FUNCTION public.diesel_manage_updated_at(_tbl regclass) OWNER TO postgres;

--
-- Name: diesel_set_updated_at(); Type: FUNCTION; Schema: public; Owner: postgres
--

CREATE FUNCTION public.diesel_set_updated_at() RETURNS trigger
    LANGUAGE plpgsql
    AS $$
BEGIN
    IF (
        NEW IS DISTINCT FROM OLD AND
        NEW.updated_at IS NOT DISTINCT FROM OLD.updated_at
    ) THEN
        NEW.updated_at := current_timestamp;
    END IF;
    RETURN NEW;
END;
$$;


ALTER FUNCTION public.diesel_set_updated_at() OWNER TO postgres;

SET default_tablespace = '';

SET default_table_access_method = heap;

--
-- Name: __diesel_schema_migrations; Type: TABLE; Schema: public; Owner: postgres
--

CREATE TABLE public.__diesel_schema_migrations (
    version character varying(50) NOT NULL,
    run_on timestamp without time zone DEFAULT CURRENT_TIMESTAMP NOT NULL
);


ALTER TABLE public.__diesel_schema_migrations OWNER TO postgres;

--
-- Name: friend_requests; Type: TABLE; Schema: public; Owner: postgres
--

CREATE TABLE public.friend_requests (
    id bigint NOT NULL,
    sender_id uuid NOT NULL,
    receiver_id uuid NOT NULL,
    message character varying,
    created_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP NOT NULL,
    CONSTRAINT friend_requests_check CHECK ((sender_id <> receiver_id))
);


ALTER TABLE public.friend_requests OWNER TO postgres;

--
-- Name: friend_requests_id_seq; Type: SEQUENCE; Schema: public; Owner: postgres
--

CREATE SEQUENCE public.friend_requests_id_seq
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


ALTER TABLE public.friend_requests_id_seq OWNER TO postgres;

--
-- Name: friend_requests_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: postgres
--

ALTER SEQUENCE public.friend_requests_id_seq OWNED BY public.friend_requests.id;


--
-- Name: friends; Type: TABLE; Schema: public; Owner: postgres
--

CREATE TABLE public.friends (
    id bigint NOT NULL,
    user1_id uuid NOT NULL,
    user2_id uuid NOT NULL,
    created_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP NOT NULL,
    CONSTRAINT friends_check CHECK ((user2_id > user1_id))
);


ALTER TABLE public.friends OWNER TO postgres;

--
-- Name: friends_id_seq; Type: SEQUENCE; Schema: public; Owner: postgres
--

CREATE SEQUENCE public.friends_id_seq
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


ALTER TABLE public.friends_id_seq OWNER TO postgres;

--
-- Name: friends_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: postgres
--

ALTER SEQUENCE public.friends_id_seq OWNED BY public.friends.id;


--
-- Name: google_users; Type: TABLE; Schema: public; Owner: postgres
--

CREATE TABLE public.google_users (
    id character varying NOT NULL,
    user_id uuid NOT NULL,
    created_at timestamp without time zone DEFAULT CURRENT_TIMESTAMP NOT NULL
);


ALTER TABLE public.google_users OWNER TO postgres;

--
-- Name: peers; Type: TABLE; Schema: public; Owner: postgres
--

CREATE TABLE public.peers (
    peer_id uuid NOT NULL,
    user_id uuid NOT NULL,
    created_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP NOT NULL
);


ALTER TABLE public.peers OWNER TO postgres;

--
-- Name: users; Type: TABLE; Schema: public; Owner: postgres
--

CREATE TABLE public.users (
    id uuid DEFAULT gen_random_uuid() NOT NULL,
    user_name character varying NOT NULL,
    name character varying NOT NULL,
    nick_name character varying,
    given_name character varying,
    middle_name character varying,
    family_name character varying,
    email character varying NOT NULL,
    locale character varying DEFAULT 'en'::character varying NOT NULL,
    verified_email boolean DEFAULT false NOT NULL,
    picture character varying,
    created_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP NOT NULL,
    updated_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP NOT NULL
);


ALTER TABLE public.users OWNER TO postgres;

--
-- Name: friend_requests id; Type: DEFAULT; Schema: public; Owner: postgres
--

ALTER TABLE ONLY public.friend_requests ALTER COLUMN id SET DEFAULT nextval('public.friend_requests_id_seq'::regclass);


--
-- Name: friends id; Type: DEFAULT; Schema: public; Owner: postgres
--

ALTER TABLE ONLY public.friends ALTER COLUMN id SET DEFAULT nextval('public.friends_id_seq'::regclass);


--
-- Data for Name: __diesel_schema_migrations; Type: TABLE DATA; Schema: public; Owner: postgres
--

COPY public.__diesel_schema_migrations (version, run_on) FROM stdin;
00000000000000	2023-04-25 11:33:54.177185
20220206103033	2023-04-25 11:33:54.184053
20220616095715	2023-04-25 11:33:54.235865
20230314101254	2023-04-25 11:33:54.255361
\.


--
-- Data for Name: friend_requests; Type: TABLE DATA; Schema: public; Owner: postgres
--

COPY public.friend_requests (id, sender_id, receiver_id, message, created_at) FROM stdin;
\.


--
-- Data for Name: friends; Type: TABLE DATA; Schema: public; Owner: postgres
--

COPY public.friends (id, user1_id, user2_id, created_at) FROM stdin;
\.


--
-- Data for Name: google_users; Type: TABLE DATA; Schema: public; Owner: postgres
--

COPY public.google_users (id, user_id, created_at) FROM stdin;
108332985460156157078	ca45a857-dec2-436c-868a-50986f53394e	2023-04-25 12:28:27.86294
\.


--
-- Data for Name: peers; Type: TABLE DATA; Schema: public; Owner: postgres
--

COPY public.peers (peer_id, user_id, created_at) FROM stdin;
\.


--
-- Data for Name: users; Type: TABLE DATA; Schema: public; Owner: postgres
--

COPY public.users (id, user_name, name, nick_name, given_name, middle_name, family_name, email, locale, verified_email, picture, created_at, updated_at) FROM stdin;
ca45a857-dec2-436c-868a-50986f53394e	Reinhard Bronner	Reinhard Bronner	\N	Reinhard	\N	Bronner	womakid@gmail.com	en	f	https://lh3.googleusercontent.com/a/AGNmyxY3ZMIKb3ZizNIMuJwYDPEtCl3iKUPAh5DWoG1GIA=s96-c	2023-04-25 12:28:27.86294+00	2023-04-25 12:28:27.86294+00
\.


--
-- Name: friend_requests_id_seq; Type: SEQUENCE SET; Schema: public; Owner: postgres
--

SELECT pg_catalog.setval('public.friend_requests_id_seq', 1, false);


--
-- Name: friends_id_seq; Type: SEQUENCE SET; Schema: public; Owner: postgres
--

SELECT pg_catalog.setval('public.friends_id_seq', 1, false);


--
-- Name: __diesel_schema_migrations __diesel_schema_migrations_pkey; Type: CONSTRAINT; Schema: public; Owner: postgres
--

ALTER TABLE ONLY public.__diesel_schema_migrations
    ADD CONSTRAINT __diesel_schema_migrations_pkey PRIMARY KEY (version);


--
-- Name: friend_requests friend_requests_pkey; Type: CONSTRAINT; Schema: public; Owner: postgres
--

ALTER TABLE ONLY public.friend_requests
    ADD CONSTRAINT friend_requests_pkey PRIMARY KEY (id);


--
-- Name: friend_requests friend_requests_sender_id_receiver_id_key; Type: CONSTRAINT; Schema: public; Owner: postgres
--

ALTER TABLE ONLY public.friend_requests
    ADD CONSTRAINT friend_requests_sender_id_receiver_id_key UNIQUE (sender_id, receiver_id);


--
-- Name: friends friends_pkey; Type: CONSTRAINT; Schema: public; Owner: postgres
--

ALTER TABLE ONLY public.friends
    ADD CONSTRAINT friends_pkey PRIMARY KEY (id);


--
-- Name: friends friends_user1_id_user2_id_key; Type: CONSTRAINT; Schema: public; Owner: postgres
--

ALTER TABLE ONLY public.friends
    ADD CONSTRAINT friends_user1_id_user2_id_key UNIQUE (user1_id, user2_id);


--
-- Name: google_users google_users_pkey; Type: CONSTRAINT; Schema: public; Owner: postgres
--

ALTER TABLE ONLY public.google_users
    ADD CONSTRAINT google_users_pkey PRIMARY KEY (id);


--
-- Name: peers peers_pkey; Type: CONSTRAINT; Schema: public; Owner: postgres
--

ALTER TABLE ONLY public.peers
    ADD CONSTRAINT peers_pkey PRIMARY KEY (peer_id);


--
-- Name: users users_email_key; Type: CONSTRAINT; Schema: public; Owner: postgres
--

ALTER TABLE ONLY public.users
    ADD CONSTRAINT users_email_key UNIQUE (email);


--
-- Name: users users_pkey; Type: CONSTRAINT; Schema: public; Owner: postgres
--

ALTER TABLE ONLY public.users
    ADD CONSTRAINT users_pkey PRIMARY KEY (id);


--
-- Name: users users_user_name_key; Type: CONSTRAINT; Schema: public; Owner: postgres
--

ALTER TABLE ONLY public.users
    ADD CONSTRAINT users_user_name_key UNIQUE (user_name);


--
-- Name: users set_updated_at; Type: TRIGGER; Schema: public; Owner: postgres
--

CREATE TRIGGER set_updated_at BEFORE UPDATE ON public.users FOR EACH ROW EXECUTE FUNCTION public.diesel_set_updated_at();


--
-- Name: friend_requests friend_requests_receiver_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: postgres
--

ALTER TABLE ONLY public.friend_requests
    ADD CONSTRAINT friend_requests_receiver_id_fkey FOREIGN KEY (receiver_id) REFERENCES public.users(id);


--
-- Name: friend_requests friend_requests_sender_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: postgres
--

ALTER TABLE ONLY public.friend_requests
    ADD CONSTRAINT friend_requests_sender_id_fkey FOREIGN KEY (sender_id) REFERENCES public.users(id);


--
-- Name: friends friends_user1_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: postgres
--

ALTER TABLE ONLY public.friends
    ADD CONSTRAINT friends_user1_id_fkey FOREIGN KEY (user1_id) REFERENCES public.users(id);


--
-- Name: friends friends_user2_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: postgres
--

ALTER TABLE ONLY public.friends
    ADD CONSTRAINT friends_user2_id_fkey FOREIGN KEY (user2_id) REFERENCES public.users(id);


--
-- Name: google_users google_users_user_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: postgres
--

ALTER TABLE ONLY public.google_users
    ADD CONSTRAINT google_users_user_id_fkey FOREIGN KEY (user_id) REFERENCES public.users(id);


--
-- Name: peers peers_user_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: postgres
--

ALTER TABLE ONLY public.peers
    ADD CONSTRAINT peers_user_id_fkey FOREIGN KEY (user_id) REFERENCES public.users(id);


--
-- PostgreSQL database dump complete
--

