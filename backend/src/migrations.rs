use tokio_postgres::Client;

struct Migration {
    version: i32,
    name: &'static str,
    up: &'static str,
    down: &'static str,
}

const MIGRATIONS: &[Migration] = &[
    Migration {
        version: 1,
        name: "create_users",
        up: r#"
            CREATE TABLE IF NOT EXISTS users (
                id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
                username VARCHAR(100) UNIQUE NOT NULL,
                password_hash TEXT NOT NULL,
                display_name VARCHAR(200) NOT NULL DEFAULT '',
                role VARCHAR(50) NOT NULL DEFAULT 'admin',
                created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
            );
            -- Default admin user (password: admin123 - CHANGE IN PRODUCTION)
            INSERT INTO users (username, password_hash, display_name, role)
            VALUES ('admin', '$2b$12$LJ3m4ys3Lk0TSwHCbMIJfuMDc8LMEj7MqC9VlAEwS6OqGbGYOOhW6', 'Administrator', 'admin')
            ON CONFLICT (username) DO NOTHING;
        "#,
        down: "DROP TABLE IF EXISTS users CASCADE;",
    },
    Migration {
        version: 2,
        name: "create_pages",
        up: r#"
            CREATE TABLE IF NOT EXISTS pages (
                id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
                title VARCHAR(500) NOT NULL,
                slug VARCHAR(500) UNIQUE NOT NULL,
                content TEXT NOT NULL DEFAULT '',
                excerpt TEXT NOT NULL DEFAULT '',
                meta_title VARCHAR(500) NOT NULL DEFAULT '',
                meta_description TEXT NOT NULL DEFAULT '',
                tags TEXT[] NOT NULL DEFAULT '{}',
                is_visible BOOLEAN NOT NULL DEFAULT true,
                is_pinned BOOLEAN NOT NULL DEFAULT false,
                sort_order INTEGER NOT NULL DEFAULT 0,
                created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
                updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
            );
            CREATE INDEX IF NOT EXISTS idx_pages_slug ON pages(slug);
            CREATE INDEX IF NOT EXISTS idx_pages_visible ON pages(is_visible);
            CREATE INDEX IF NOT EXISTS idx_pages_tags ON pages USING GIN(tags);
            CREATE INDEX IF NOT EXISTS idx_pages_sort ON pages(is_pinned DESC, sort_order ASC);
        "#,
        down: "DROP TABLE IF EXISTS pages CASCADE;",
    },
    Migration {
        version: 3,
        name: "create_revisions",
        up: r#"
            CREATE TABLE IF NOT EXISTS revisions (
                id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
                page_id UUID NOT NULL REFERENCES pages(id) ON DELETE CASCADE,
                title VARCHAR(500) NOT NULL,
                content TEXT NOT NULL DEFAULT '',
                excerpt TEXT NOT NULL DEFAULT '',
                meta_title VARCHAR(500) NOT NULL DEFAULT '',
                meta_description TEXT NOT NULL DEFAULT '',
                tags TEXT[] NOT NULL DEFAULT '{}',
                revision_number INTEGER NOT NULL DEFAULT 1,
                created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
            );
            CREATE INDEX IF NOT EXISTS idx_revisions_page ON revisions(page_id, revision_number DESC);
        "#,
        down: "DROP TABLE IF EXISTS revisions CASCADE;",
    },
    Migration {
        version: 4,
        name: "create_sliders",
        up: r#"
            CREATE TABLE IF NOT EXISTS sliders (
                id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
                title VARCHAR(500) NOT NULL,
                description TEXT NOT NULL DEFAULT '',
                image_url TEXT NOT NULL DEFAULT '',
                link_url TEXT NOT NULL DEFAULT '',
                is_visible BOOLEAN NOT NULL DEFAULT true,
                is_pinned BOOLEAN NOT NULL DEFAULT false,
                sort_order INTEGER NOT NULL DEFAULT 0,
                created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
                updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
            );
        "#,
        down: "DROP TABLE IF EXISTS sliders CASCADE;",
    },
    Migration {
        version: 5,
        name: "create_media",
        up: r#"
            CREATE TABLE IF NOT EXISTS media (
                id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
                filename VARCHAR(500) NOT NULL,
                original_name VARCHAR(500) NOT NULL,
                mime_type VARCHAR(100) NOT NULL,
                size_bytes BIGINT NOT NULL DEFAULT 0,
                width INTEGER,
                height INTEGER,
                url TEXT NOT NULL,
                created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
            );
        "#,
        down: "DROP TABLE IF EXISTS media CASCADE;",
    },
    Migration {
        version: 6,
        name: "create_analytics",
        up: r#"
            CREATE TABLE IF NOT EXISTS page_visits (
                id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
                page_path TEXT NOT NULL,
                referrer TEXT,
                user_agent TEXT,
                ip_hash VARCHAR(64) NOT NULL DEFAULT '',
                visited_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
            );
            CREATE INDEX IF NOT EXISTS idx_visits_path ON page_visits(page_path);
            CREATE INDEX IF NOT EXISTS idx_visits_date ON page_visits(visited_at);
            CREATE INDEX IF NOT EXISTS idx_visits_ip ON page_visits(ip_hash);
        "#,
        down: "DROP TABLE IF EXISTS page_visits CASCADE;",
    },
    Migration {
        version: 7,
        name: "create_captchas",
        up: r#"
            CREATE TABLE IF NOT EXISTS captchas (
                id VARCHAR(100) PRIMARY KEY,
                answer VARCHAR(20) NOT NULL,
                created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
            );
        "#,
        down: "DROP TABLE IF EXISTS captchas CASCADE;",
    },
    Migration {
        version: 8,
        name: "create_site_settings",
        up: r#"
            CREATE TABLE IF NOT EXISTS site_settings (
                id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
                site_name VARCHAR(200) NOT NULL DEFAULT 'CMS',
                site_description TEXT NOT NULL DEFAULT '',
                logo_url TEXT NOT NULL DEFAULT '',
                favicon_url TEXT NOT NULL DEFAULT '',
                footer_text TEXT NOT NULL DEFAULT '',
                social_links JSONB NOT NULL DEFAULT '{}',
                custom_head_html TEXT NOT NULL DEFAULT '',
                updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
            );
            INSERT INTO site_settings (site_name, site_description)
            VALUES ('My CMS', 'A modern content management system')
            ON CONFLICT DO NOTHING;
        "#,
        down: "DROP TABLE IF EXISTS site_settings CASCADE;",
    },
    Migration {
        version: 9,
        name: "create_menus",
        up: r#"
            CREATE TABLE IF NOT EXISTS menus (
                id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
                label VARCHAR(200) NOT NULL,
                url TEXT NOT NULL DEFAULT '#',
                parent_id UUID REFERENCES menus(id) ON DELETE SET NULL,
                sort_order INTEGER NOT NULL DEFAULT 0,
                is_visible BOOLEAN NOT NULL DEFAULT true,
                open_in_new_tab BOOLEAN NOT NULL DEFAULT false,
                created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
            );
            CREATE INDEX IF NOT EXISTS idx_menus_parent ON menus(parent_id);
            CREATE INDEX IF NOT EXISTS idx_menus_sort ON menus(sort_order ASC);
        "#,
        down: "DROP TABLE IF EXISTS menus CASCADE;",
    },
    Migration {
        version: 10,
        name: "create_redirects",
        up: r#"
            CREATE TABLE IF NOT EXISTS redirects (
                id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
                from_path TEXT NOT NULL UNIQUE,
                to_path TEXT NOT NULL,
                status_code INTEGER NOT NULL DEFAULT 301,
                created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
            );
        "#,
        down: "DROP TABLE IF EXISTS redirects CASCADE;",
    },
    Migration {
        version: 11,
        name: "create_fulltext_search",
        up: r#"
            ALTER TABLE pages ADD COLUMN IF NOT EXISTS search_vector tsvector;
            CREATE INDEX IF NOT EXISTS idx_pages_search ON pages USING GIN(search_vector);

            CREATE OR REPLACE FUNCTION pages_search_trigger() RETURNS trigger AS $$
            BEGIN
                NEW.search_vector :=
                    setweight(to_tsvector('simple', COALESCE(NEW.title, '')), 'A') ||
                    setweight(to_tsvector('simple', COALESCE(NEW.excerpt, '')), 'B') ||
                    setweight(to_tsvector('simple', COALESCE(NEW.content, '')), 'C');
                RETURN NEW;
            END
            $$ LANGUAGE plpgsql;

            DROP TRIGGER IF EXISTS tsvector_update ON pages;
            CREATE TRIGGER tsvector_update BEFORE INSERT OR UPDATE ON pages
                FOR EACH ROW EXECUTE FUNCTION pages_search_trigger();

            -- Update existing rows
            UPDATE pages SET search_vector =
                setweight(to_tsvector('simple', COALESCE(title, '')), 'A') ||
                setweight(to_tsvector('simple', COALESCE(excerpt, '')), 'B') ||
                setweight(to_tsvector('simple', COALESCE(content, '')), 'C');
        "#,
        down: r#"
            DROP TRIGGER IF EXISTS tsvector_update ON pages;
            DROP FUNCTION IF EXISTS pages_search_trigger();
            ALTER TABLE pages DROP COLUMN IF EXISTS search_vector;
        "#,
    },
    Migration {
        version: 12,
        name: "create_csrf_tokens",
        up: r#"
            CREATE TABLE IF NOT EXISTS csrf_tokens (
                token VARCHAR(128) PRIMARY KEY,
                created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
                expires_at TIMESTAMPTZ NOT NULL DEFAULT NOW() + INTERVAL '2 hours'
            );
            CREATE INDEX IF NOT EXISTS idx_csrf_expires ON csrf_tokens(expires_at);
        "#,
        down: "DROP TABLE IF EXISTS csrf_tokens CASCADE;",
    },
];

pub async fn run_migrations(client: &Client) -> Result<(), tokio_postgres::Error> {
    // Create migrations tracking table
    client.execute(
        "CREATE TABLE IF NOT EXISTS _migrations (
            version INTEGER PRIMARY KEY,
            name VARCHAR(200) NOT NULL,
            applied_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
        )",
        &[],
    ).await?;

    for migration in MIGRATIONS {
        let exists = client.query_opt(
            "SELECT version FROM _migrations WHERE version = $1",
            &[&migration.version],
        ).await?;

        if exists.is_none() {
            tracing::info!("Running migration {}: {}", migration.version, migration.name);
            client.batch_execute(migration.up).await?;
            client.execute(
                "INSERT INTO _migrations (version, name) VALUES ($1, $2)",
                &[&migration.version, &migration.name],
            ).await?;
        }
    }

    Ok(())
}

pub async fn _rollback_migration(client: &Client, target_version: i32) -> Result<(), tokio_postgres::Error> {
    let mut versions: Vec<i32> = Vec::new();
    let rows = client.query(
        "SELECT version FROM _migrations WHERE version > $1 ORDER BY version DESC",
        &[&target_version],
    ).await?;

    for row in &rows {
        versions.push(row.get(0));
    }

    for version in versions {
        if let Some(migration) = MIGRATIONS.iter().find(|m| m.version == version) {
            tracing::info!("Rolling back migration {}: {}", migration.version, migration.name);
            client.batch_execute(migration.down).await?;
            client.execute(
                "DELETE FROM _migrations WHERE version = $1",
                &[&migration.version],
            ).await?;
        }
    }

    Ok(())
}
