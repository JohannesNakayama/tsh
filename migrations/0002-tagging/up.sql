create table if not exists zettel_tag (
    zettel_id  integer not null references zettel(id)
  , tag        text    not null
  , created_at integer not null default (unixepoch('subsec') * 1000)
  , unique(zettel_id, tag)
);


