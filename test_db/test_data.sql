COPY public.users (id, user_name, display_name, email, locale, verified_email, created_at, updated_at) FROM stdin;
ca45a857-dec2-436c-868a-50986f53394e	Reinhard Bronner	Reinhard Bronner	womakid@gmail.com	en	f	2023-04-25 12:28:27.86294+00	2023-04-25 12:28:27.86294+00
ac4758f7-1b76-4cf3-a8d9-dd0df25672f0	test-user	Test	test@user.com	de-DE	f	2023-04-25 14:06:21.294383+00	2023-04-25 14:06:21.294383+00
\.

COPY public.google_users (id, user_id, created_at) FROM stdin;
108332985460156157078	ca45a857-dec2-436c-868a-50986f53394e	2023-04-25 12:28:27.86294
\.

--
-- Data for Name: friend_requests; Type: TABLE DATA; Schema: public; Owner: postgres
--

COPY public.friend_requests (id, sender_id, receiver_id, message, created_at) FROM stdin;
1	ca45a857-dec2-436c-868a-50986f53394e	ac4758f7-1b76-4cf3-a8d9-dd0df25672f0	AH	2023-04-25 15:36:33.098617+00
\.

--
-- Name: friend_requests_id_seq; Type: SEQUENCE SET; Schema: public; Owner: postgres
--

SELECT pg_catalog.setval('public.friend_requests_id_seq', 1, true);