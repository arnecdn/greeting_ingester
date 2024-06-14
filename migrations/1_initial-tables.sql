-- Add migration script here
CREATE TABLE IF NOT EXISTS greeting
(
    id         BIGINT GENERATED ALWAYS AS IDENTITY PRIMARY KEY ,
    message_id UUID         NOT NULL,
    "from"     VARCHAR(255) NOT NULL,
    "to"       VARCHAR(255) NOT NULL,
    heading    VARCHAR(255) NOT NULL,
    message    VARCHAR(255) NOT NULL,
    created    TIMESTAMP    NOT NULL
);