-- Add up migration script here
CREATE TYPE account_status AS ENUM ('active', 'inactive', 'suspended');
COMMENT ON TYPE account_status IS '枚举类型，表示账号状态';

CREATE TYPE language AS ENUM ('en-US', 'zh-CN', 'fr-FR', 'es-ES');
COMMENT ON TYPE language IS '枚举类型，表示系统语言选项';
