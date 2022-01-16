drop table if exists event_update_valid_block;
create table event_update_valid_block
(
    chain_id bigint not null,
    cid text not null,
    donor text not null,
    update_block bigint not null,
    end_block bigint not null,
    -- provider_id bigint not null,
    ts timestamp without time zone,
    primary key (update_block, donor, end_block, cid, chain_id)
);

drop table if exists pinned_cids;
create table pinned_cids
(
    chain_id bigint not null,
    node text NOT NULL,
    cid TEXT NOT NULL,
    end_block BIGINT NOT NULL,
    primary key (chain_id, node, cid, end_block)
);

drop table if exists failed_pins;
create table failed_pins
(
    chain_id bigint not null,
    node text NOT NULL,
    cid TEXT NOT NULL,
    end_block BIGINT NOT NULL,
    primary key (chain_id, node, cid, end_block)
);
