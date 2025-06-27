create table thought (
    id        integer primary key autoincrement
  , content   text    not null
);

create table edge (
    node_id   integer not null references thought(id)
  , parent_id integer          references thought(id) default null -- nullable
  , primary key (node_id, parent_id)
);
