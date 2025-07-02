create table zettel (
    id         integer primary key
  , content    text    not null
  , created_at integer not null default (unixepoch('subsec') * 1000)
);

create virtual table zettel_embedding using vec0(
    zettel_id integer    not null references zettel(id)
  , embedding  float[384] -- embedding shouldn't be null (but not null constraint doesn't work with sqlite-vec)
);

create table zettel_edge (
    node_id   integer not null references zettel(id)
  , parent_id integer          references zettel(id) default null -- nullable
  , primary key (node_id, parent_id)
);

