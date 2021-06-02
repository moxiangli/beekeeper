tables
create table `host_online_find` (
    data_id varchar(64) not null,
    host_ip varchar(64) not null,
    scan_time datetime not null,
    scan_port int not null,
    primary key (data_id)
) comment 'find online host list.'

create table `host_swarm_list` (
    host_ip varchar(64) not null,
    host_status int not null,
    update_time datetime not null,
    primary key (host_ip)
) comment 'host in swarm.'

create table `host_operate_history` (
    data_id varchar(64) not null,
    host_ip varchar(64) not null,
    operator_id varchar(64) not null,
    operate_type varchar(32) not null,
    operate_time datetime not null,
    operate_reason varchar(256) default null,
    primary key (data_id)
) comment 'host add or remove record.'

create table `swarm_operate_history` (
    data_id varchar(64) not null,
    host_ip varchar(64) not null,
    operator_id varchar(64) not null,
    operate_type varchar(32) not null,
    operate_time datetime not null,
    operate_reason varchar(256) default null,
    primary key (data_id)
) comment 'swarm docker start or stop history.'