-- Add migration script here
create sequence logg_id_seq;
create table logg
(
    id           bigint                   default nextval('logg_id_seq'::regclass) not null,
    greeting_id  bigint,
    opprettet    timestamp with time zone default CURRENT_TIMESTAMP                not null,
    primary key (id),
    constraint logg_dokument_id_type_fkey
        foreign key ( greeting_id) references greeting
);


create unique index logg_dokument_id_idx
    on logg (greeting_id);

create table ikke_paa_logg
(
    greeting_id  bigint
);

CREATE UNIQUE INDEX ikke_paa_logg_dok_id
    ON ikke_paa_logg (greeting_id);

CREATE OR REPLACE FUNCTION generate_logg()
    RETURNS VOID
    LANGUAGE plpgsql
AS
$$
DECLARE
lock_id CONSTANT INTEGER := 42424242;
    lock_succeeded   boolean default false;
BEGIN
    -- Try to obtain exclusive transaction lock.
    -- The transaction lock will be automatically released when transaction ends.
    -- That happens when stored function ends since all stored functions
    -- in PostgreSQL is committed when it ends.
SELECT pg_try_advisory_xact_lock(lock_id)
INTO lock_succeeded;

IF lock_succeeded THEN
        WITH deletions AS (
               DELETE
                 FROM ikke_paa_logg
            RETURNING greeting_id
        )
        INSERT
          INTO logg (greeting_id)
SELECT greeting_id

FROM deletions
ORDER BY greeting_id ASC;
END IF;
END;
$$;

