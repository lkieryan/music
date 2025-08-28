# AGENTS.md

This file provides guidance to agents when working with code in this repository.

## Critical Task Management Rule

**IMPORTANT**: Due to limited context length, you MUST follow this task management process:

### Before Implementation - Create Implementation Plan
Before starting any task, you MUST create a detailed implementation plan document that includes:

1. **User Requirements**: Clearly state what the user requested
2. **Implementation Approach**: How you plan to complete the task
3. **Implementation Ideas**: Your technical approach and reasoning
4. **Task Breakdown**: Break down the task into major and sub-tasks

### Task Management Format
Use the following format for tracking tasks:

- `[ ] Task Name` - Represents incomplete tasks
- `[x] Task Name` - Represents completed tasks

### Task Structure
```
## Major Task 1
- [ ] Sub-task 1.1
- [ ] Sub-task 1.2
- [x] Sub-task 1.3 (completed)

## Major Task 2
- [ ] Sub-task 2.1
- [ ] Sub-task 2.2
```

### Implementation Process
1. **Create Plan**: Write detailed implementation plan before starting
2. **Get Approval**: Wait for user to approve the plan
3. **Execute Tasks**: Work through tasks systematically
4. **Update Status**: Mark tasks as `[x] completed` after user verification
5. **Document Progress**: Keep implementation plan updated with progress

### Example Implementation Plan Structure
```markdown
# Implementation Plan: [Task Name]

## User Requirements
[Detailed description of what user requested]

## Implementation Approach
[How you plan to complete the task]

## Implementation Ideas
[Technical approach and reasoning]

## Task Breakdown

## Major Task 1: [Task Description]
- [ ] Sub-task 1.1: [Description]
- [ ] Sub-task 1.2: [Description]
- [ ] Sub-task 1.3: [Description]

## Major Task 2: [Task Description]
- [ ] Sub-task 2.1: [Description]
- [ ] Sub-task 2.2: [Description]
```

**Remember**: Always create an implementation plan before starting any task, and keep task status updated throughout the implementation process!

## Implementation Plan Documentation Management

**IMPORTANT**: All implementation plans must be stored in a structured folder system with specific naming and organization rules:

### Documentation Storage Structure
Create a folder named `implementation-plans/` in the project root with the following structure:
```
implementation-plans/
├── pending/           # Pending task documents
└── completed/         # Completed task documents
```

### Document Naming Convention
- **Format**: `YYYY-MM-DD-HHMM-RequirementName.md`
- **Example**: `2025-09-04-1430-MusicPlayerUIOptimization.md`
- **Requirements**:
  - Use 24-hour format for time (HHMM)
  - Use descriptive English names for requirements
  - No spaces in the filename (use underscores if needed)

### Document Content Requirements
- **Language**: All content MUST be written in Chinese
- **Structure**: Must follow the implementation plan structure specified above
- **Encoding**: UTF-8 to support Chinese characters

### Document Management Process

#### For New Tasks
1. **Create Document**: When starting a new task, create a new document in `implementation-plans/pending/`
2. **Name Document**: Use the naming convention: `YYYY-MM-DD-HHMM-RequirementName.md`
3. **Write Content**: Write the complete implementation plan in Chinese
4. **Get Approval**: Wait for user approval before starting implementation

#### During Implementation
1. **Update Progress**: Keep the document updated with current progress
2. **Mark Completed**: Update task status from `[ ]` to `[x]` as sub-tasks are completed
3. **Maintain Chinese**: Ensure all updates and comments remain in Chinese

#### After Completion
1. **Final Update**: Update the document to show all tasks as completed
2. **Move Document**: Move the document from `pending/` to `completed/`
3. **Archive**: The document serves as a record of completed work

### Git Exclusion
- **IMPORTANT**: The entire `implementation-plans/` folder MUST be excluded from Git
- Add the following to `.gitignore`:
  ```
  # Implementation plans (temporary working documents)
  implementation-plans/
  ```

### Folder Organization Example
```
implementation-plans/
├── pending/
│   ├── 2025-09-04-1430-MusicPlayerUIOptimization.md
│   ├── 2025-09-04-1500-AddNewMusicProviderSupport.md
│   └── 2025-09-04-1530-FixAudioPlaybackIssue.md
└── completed/
    ├── 2025-09-03-1400-DatabasePerformanceOptimization.md
    ├── 2025-09-03-1430-UserAuthenticationSystem.md
    └── 2025-09-04-1300-ThemeSystemUpgrade.md
```

### Document Template
```markdown
# 实施计划: [Task Name]

## 用户需求
[用户请求的详细描述]

## 实施方法
[计划如何完成任务的描述]

## 实施思路
[技术方法和理由的说明]

## 任务分解

## 主要任务 1: [任务描述]
- [ ] 子任务 1.1: [描述]
- [ ] 子任务 1.2: [描述]
- [ ] 子任务 1.3: [描述]

## 主要任务 2: [任务描述]
- [ ] 子任务 2.1: [描述]
- [ ] 子任务 2.2: [描述]
```

**Remember**: Always create implementation plan documents in Chinese, follow the naming convention, and manage them in the proper folder structure!

## Build Commands

- `pnpm gen:types` - Generate TypeScript bindings from Rust types (required after changing types in crates/types)
- `pnpm dev` - Start Tauri development server with hot reload
- `pnpm build` - Build the application for current platform
- `pnpm build:android` - Build Android version (requires Android SDK)
- `pnpm web:dev` - Start only frontend development server
- `pnpm web:build` - Build only frontend (TypeScript compilation + Vite build)
- `pnpm lint` - Run ESLint checks

## Project Architecture

### Core Technology Stack
- **Frontend**: React 19 + TypeScript 5.8 + Vite 7 + React Router 7
- **Backend**: Tauri 2.7 + Rust with custom crates
- **State Management**: Jotai (atomic state) + Zustand (complex state)
- **UI Framework**: Radix UI Themes + Primitives + Headless UI
- **Styling**: TailwindCSS 3.4 + CSS Custom Properties + Tailwind Variants
- **Database**: SQLite + Diesel ORM with migrations

### Plugin System
- All music providers must implement `AudioProvider` trait from music-plugin-sdk
- Plugins are identified by UUID, not names
- Plugin communication uses `PluginEvent` with timeout (default 5s)
- Plugin capabilities are declared via `PluginCapability` enum
- Multi-provider support: Spotify, YouTube, Bilibili

### Type System Architecture
- Types shared between Rust and TypeScript via `crates/types` - no duplication allowed
- SDK types (plugin) and app types (internal) are separate - adapter pattern required
- TypeScript bindings auto-generated via ts-rs - manual editing will be overwritten
- Database models in `types` crate with `db` feature - separates concerns
- Frontend cannot use SDK types directly - must go through adapter conversion

### Type Conversion
- SDK types (from plugins) must be converted to app types using `src-tauri/src/music/adapter.rs`
- Never use SDK types directly in the frontend - always convert through adapter
- Use `sdk_track_to_track()` for converting plugin tracks to app tracks
- Use `app_track_to_sdk_track()` for converting app tracks back to SDK format

### Music Provider Selection
- Use `Selection` enum (All/Single/Many) for provider selection
- Provider preferences stored in `prefs.music.source` configuration
- Always check provider capabilities with `supports()` before using
- Stream URLs resolved on-demand, not stored - reduces storage and handles expiration

### Error Handling
- Use `types::errors::Result<T>` for all public functions
- Plugin operations must handle timeouts gracefully (5-second default)
- Use `tokio::task::spawn_blocking` for potentially blocking plugin operations
- Plugin event handling requires spawn_blocking for synchronous plugin methods

### Database Patterns
- Database migrations are in `crates/database/migrations/` with date-prefixed directories
- Use Diesel ORM for all database operations
- Database access is through `database` crate, not direct SQL
- Plugin state persisted in database, not in-memory - survives restarts

## Component Development Guidelines

### Before Implementation - Component Reuse Strategy
**CRITICAL**: Before implementing any new component or feature, ALWAYS check existing codebase first:

1. **Search Existing Components**:
   - Check `src/components/ui/` for reusable primitives
   - Look in `src/components/common/` for shared components
   - Review `src/hooks/` for existing custom hooks
   - Check `src/lib/` for utility functions

2. **Extend Rather Than Recreate**:
   - Extend existing components with new props/variants
   - Compose existing primitives into new combinations
   - Add new variants to existing component systems
   - Reuse existing hooks and utilities

### Component Patterns
```tsx
// Use functional components with proper hooks order
export const ComponentName = () => {
  // 1. State hooks (useState, useAtom, etc.)
  const [state, setState] = useState()
  const value = useAtomValue(someAtom)
  
  // 2. Effect hooks
  useEffect(() => {}, [])
  
  // 3. Custom hooks
  const isMobile = useMobile()
  const { t } = useTranslation()
  
  // 4. Memoized values and callbacks
  const memoizedValue = useMemo(() => computation, [deps])
  const handleClick = useCallback(() => {}, [])
  
  // 5. Render
  return (
    <div className="tailwind-classes">
      {/* Content */}
    </div>
  )
}
```

### State Management Patterns
- Jotai atoms for simple state
- Settings atoms with enhanced support via `createSettingAtom`
- Zustand for complex state
- Immer for immutable updates

### Styling Guidelines
- Use semantic CSS custom properties: `bg-background text-text border-border`
- Theme-aware styling with CSS variables
- Responsive design with custom utilities
- Multiple CSS layers: base, components, utilities, layout, menu-desktop

## Internationalization (i18n)

**CRITICAL**: All user-facing text MUST be internationalized - no hardcoded strings in components!

### Directory Structure
```
locales/
├── app/           # Application-specific texts (menus, navigation)
├── common/        # Common words and phrases (buttons, actions)
└── settings/      # Settings and configuration texts
```

### Internationalization Rules
1. **Mandatory Internationalization**:
   ```tsx
   // ❌ BAD: Hardcoded text
   <button>Save</button>
   <h1>Music Player</h1>
   
   // ✅ GOOD: Internationalized text
   <button>{t('common:words.save')}</button>
   <h1>{t('app:title')}</h1>
   ```

2. **Namespace Organization**:
   - `app` - Application-specific content (page titles, menu items)
   - `common` - Reusable words and phrases (ok, cancel, save, etc.)
   - `settings` - Settings panel and configuration texts

3. **Key Naming Convention**:
   ```json
   {
     "menu.home": "Home",
     "words.save": "Save",
     "actions.delete": "Delete",
     "tips.load-error": "Failed to load"
   }
   ```

4. **Alphabetical Sorting**:
   - **MANDATORY**: All keys in JSON files MUST be sorted alphabetically (a-z)
   - This improves maintainability and prevents duplicate keys

5. **Language Support**:
   - **Primary**: `zh-CN` (Chinese Simplified)
   - **Secondary**: `en-US` (English)
   - **Fallback Strategy**: zh-CN as fallback, zh-TW → zh-CN

## Git Commit Rules

### Commit Message Format
```
<type>[optional scope]: <description>

[optional body]

[optional footer(s)]
```

### Commit Types
- `feat` - A new feature
- `fix` - A bug fix
- `docs` - Documentation only changes
- `style` - Changes that do not affect the meaning of the code
- `refactor` - A code change that neither fixes a bug nor adds a feature
- `perf` - A code change that improves performance
- `test` - Adding missing tests or correcting existing tests
- `build` - Changes that affect the build system or external dependencies
- `ci` - Changes to our CI configuration files and scripts
- `chore` - Other changes that don't modify src or test files
- `revert` - Reverts a previous commit

### General Principles
- **Language**: All commit messages MUST be in English
- **Format**: Follow conventional commit format strictly
- **Clarity**: Describe what functionality was added and what issues were fixed
- **Consistency**: Use standardized templates for different types of commits

## Testing Single Components
- Run specific crate tests: `cargo test -p <crate_name>`
- Test plugin functionality: `cargo test -p plugins`
- Test type generation: `cargo test --manifest-path crates/types/Cargo.toml --no-default-features --features ts-rs`
- Each crate has its own tests - no integration test directory
- Plugin tests require full plugin system - complex setup
- Type generation tests require specific feature flags - not default
- Database tests use test migrations - separate from production
- Frontend tests use Vite environment - not standard Jest