create table thought (
    id      integer primary key
  , content text    not null
);

create virtual table thought_embedding using vec0(
    thought_id integer    not null references thought(id)
  , embedding  float[384]
);

create table edge (
    node_id   integer not null references thought(id)
  , parent_id integer          references thought(id) default null -- nullable
  , primary key (node_id, parent_id)
);

