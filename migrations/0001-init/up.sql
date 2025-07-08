create table if not exists zettel (
    id         integer primary key
  , content    text    not null
  , created_at integer not null default (unixepoch('subsec') * 1000)
) strict;

create virtual table if not exists zettel_embedding using vec0(
    zettel_id integer    not null references zettel(id)
  , embedding float[384] -- embedding shouldn't be null (but not null constraint doesn't work with sqlite-vec)
);

create table if not exists zettel_edge (
    node_id   integer not null references zettel(id)
  , parent_id integer          references zettel(id) default null -- nullable, for root zettels
  , primary key (node_id, parent_id)
);

create table if not exists article (
    id         integer primary key
  , zettel_id  integer not null references zettel(id)
  , title      text    -- nullable, can be deferred
  , content    text    not null
  , created_at integer not null default (unixepoch('subsec') * 1000)
);

create table if not exists zettel_lineage (
    ancestor_id   integer
  , descendant_id integer not null
  , separation    integer not null
  , primary key(ancestor_id, descendant_id)
) strict;

create trigger after_insert_zettel_edge after insert on zettel_edge
when new.parent_id is not null
begin
  -- parent
  insert into zettel_lineage (ancestor_id, descendant_id, separation)
  values (new.parent_id, new.node_id, 1) on conflict do nothing;

  -- all ancestors
  insert into zettel_lineage
  select 
      ancestor_id
    , new.node_id as descendant_id
    , 1 + separation as separation
  from zettel_lineage ancestor
  where ancestor.descendant_id = new.parent_id;
end;

-- In a DAG, depth is defined as the length of the longest path from any root node to the current node.
create view if not exists zettel_depth as
with root_nodes as (
  select id
  from zettel
  except
  select node_id
  from zettel_edge
  where parent_id is not null
)
, descendant_depths as (
  select
      zl.descendant_id as zettel_id
    , max(zl.separation) as depth
  from zettel_lineage zl
  join root_nodes rn on zl.ancestor_id = rn.id
  group by zl.descendant_id
)
select
    rn.id as zettel_id
  , 0 as depth
from root_nodes rn
union all
select
    dd.zettel_id
  , dd.depth
from descendant_depths dd;
