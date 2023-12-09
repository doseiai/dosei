CREATE TABLE cron_jobs (
   uuid UUID NOT NULL,
   schedule TEXT NOT NULL,
   entrypoint TEXT NOT NULL,
   owner_id UUID NOT NULL,
   deployment_id UUID NOT NULL,
   updated_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
   created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
   PRIMARY KEY (uuid),
   UNIQUE (uuid, owner_id, deployment_id)
);
