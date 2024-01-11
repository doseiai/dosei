CREATE TABLE IF NOT EXISTS cron_jobs (
   id UUID NOT NULL,
   schedule TEXT NOT NULL,
   entrypoint TEXT NOT NULL,
   owner_id UUID NOT NULL,
   project_id UUID NOT NULL,
   --- Git Commit (sha1 hash)
   deployment_id TEXT NOT NULL,
   updated_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP NOT NULL,
   created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP NOT NULL,
   PRIMARY KEY (id)
);

CREATE TABLE IF NOT EXISTS envs (
   id UUID NOT NULL,
   name TEXT NOT NULL,
   value TEXT NOT NULL,
   project_id UUID NOT NULL,
   owner_id UUID NOT NULL,
   updated_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP NOT NULL,
   created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP NOT NULL,
   PRIMARY KEY (id),
   UNIQUE (name, project_id, owner_id)
);

DO $$
    BEGIN
        IF NOT EXISTS (SELECT 1 FROM pg_type WHERE typname = 'git_source') THEN
            CREATE TYPE git_source AS ENUM ('github', 'gitlab', 'bitbucket');
        END IF;
    END
$$;

CREATE TABLE IF NOT EXISTS projects (
    id UUID NOT NULL,
    name TEXT NOT NULL,
    owner_id UUID NOT NULL,
    git_source git_source NOT NULL,
    git_source_metadata jsonb NOT NULL,
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP NOT NULL,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP NOT NULL,
    PRIMARY KEY (id),
    UNIQUE (name, owner_id)
);

DO $$
    BEGIN
        IF NOT EXISTS (SELECT 1 FROM pg_type WHERE typname = 'deployment_status') THEN
            CREATE TYPE deployment_status AS ENUM ('queued', 'building', 'error', 'canceled', 'ready');
        END IF;
    END
$$;

CREATE TABLE IF NOT EXISTS deployments (
    id UUID NOT NULL,
    commit_id TEXT NOT NULL,
    commit_metadata jsonb NOT NULL,
    project_id UUID NOT NULL,
    owner_id UUID NOT NULL,
    status deployment_status NOT NULL,
    build_logs jsonb NOT NULL,
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP NOT NULL,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP NOT NULL,
    PRIMARY KEY (id)
);

DO $$
    BEGIN
        IF NOT EXISTS (SELECT 1 FROM pg_type WHERE typname = 'service_type') THEN
            CREATE TYPE service_type AS ENUM ('project', 'storage');
        END IF;
    END
$$;

CREATE TABLE IF NOT EXISTS domains (
   id UUID NOT NULL,
   name TEXT NOT NULL,
   service_type service_type NOT NULL,
   project_id UUID,
   storage_id UUID,
   deployment_id TEXT,
   owner_id UUID NOT NULL,
   updated_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP NOT NULL,
   created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP NOT NULL,
   PRIMARY KEY (id),
   UNIQUE (name)
);
