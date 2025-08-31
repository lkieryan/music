# AGENTS.md

This file provides guidance to agents when working with code in this repository.

## Critical Task Management Rule

**IMPORTANT**: Due to limited context length, you MUST follow this task management process:

### Before Implementation - Create Implementation Plan
Before starting any architectural task, you MUST create a detailed implementation plan document that includes:

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

**Remember**: Always create an implementation plan before starting any architectural task, and keep task status updated throughout the implementation process!

## Design Phase vs Execution Phase

### Design Phase
**Primary Purpose**: Clarify technical solutions and architecture design through diagrams and thought explanations

**Core Activities**:
- **Drawing Architecture Diagrams**: Create system architecture, component relationships, data flow diagrams
- **Explaining Design Thoughts**: Detail design approaches, technology selection rationale, and architectural decisions
- **Solution Discussions**: Discuss feasibility and optimization points with relevant stakeholders
- **Documentation Writing**: Organize design thoughts and diagrams into design documents

**Deliverables**:
- Visual documents such as architecture diagrams, flow charts, component relationship diagrams
- Detailed explanations of design thoughts and technical solutions
- Design documents stored in the `implementation-plans/design/` folder

**Characteristics**:
- No specific task breakdown or execution steps involved
- Focus on "what to do" and "why to do it this way"
- Primarily diagrams and textual descriptions, no code implementation involved
- **CRITICAL**: Design phase should focus on drawing diagrams and explaining design thinking, with task management format only used for tracking design progress

### Execution Phase
**Primary Purpose**: Transform design solutions into executable code implementation plans

**Core Activities**:
- **Task Breakdown**: Decompose overall requirements into executable major tasks and sub-tasks
- **Implementation Planning**: Create detailed implementation steps and timelines
- **Resource Allocation**: Determine required human resources, technical resources, and dependencies
- **Risk Assessment**: Identify potential risks and mitigation measures

**Deliverables**:
- Implementation plan documents stored in `implementation-plans/pending/`
- Implementation plans with detailed task breakdown
- Moved to `implementation-plans/completed/` upon completion

**Characteristics**:
- Focus on "how to do" and "when to do"
- Includes specific task breakdown and execution steps
- Provides clear guidance for subsequent coding implementation
- **CRITICAL**: Execution phase uses detailed task management format for tracking implementation progress

### Phase Transition
- **Design → Execution**: After design approval, enter execution phase and create implementation plan documents
- **Execution → Design**: Return to design phase for adjustments if design issues are discovered during execution

**Important Reminders**:
- Design phase does not involve task breakdown, only focuses on thoughts and architecture
- Execution phase is where detailed task breakdown and implementation planning occur
- Documents from both phases are stored in separate folders for easy management and tracking
- **CRITICAL**: When in design phase, focus on creating diagrams and explaining design thinking. Only move to execution phase when design is approved and detailed implementation planning is needed.

## Implementation Plan Monitoring and Requirements Management

### Real-time Monitoring
- **CRITICAL**: Must continuously monitor implementation plans in `implementation-plans/pending/` for any deviations from the original requirements
- Regularly check the progress of pending implementation plans to ensure they stay on track
- Verify that all requirements are properly understood and implemented as specified

### User Confirmation Process
- **Requirement Definition**: All requirements must be clearly defined and confirmed by the user before implementation begins
- **Progress Tracking**: When user uses "继续 xxx" (continue xxx) to resume work, must check the implementation plans in `implementation-plans/pending/` to understand current progress
- **Status Assessment**: Before taking over any task, review the corresponding implementation plan document to determine what has been completed and what remains to be done

### Requirement Clarification
- **Ambiguous Requirements**: When requirements are unclear or have multiple possible interpretations, must inform the user and ask for clarification
- **Multiple Paths**: When a requirement can be implemented in different ways, present the options to the user and let them decide
- **No Over-simplification**: Must not over-abstract or over-simplify requirements - implement exactly what the user requested
- **No Over-engineering**: Must not add unnecessary complexity or features beyond what was requested
- **Precise Implementation**: Follow the user's requirements precisely without making assumptions about what they "really meant"

### Implementation Plan Review Process
1. **Initial Review**: Before starting any task, review the implementation plan to understand the full scope
2. **Progress Check**: When resuming work with "继续 xxx", check the implementation plan to see current status
3. **Deviation Detection**: Monitor for any deviations from the original requirements during implementation
4. **User Consultation**: When in doubt or when multiple paths exist, consult with the user for direction
5. **Completion Verification**: Ensure all requirements are met before marking the task as completed

### Requirement Discussion Protocol
- **CRITICAL**: When user indicates they want to discuss requirements (e.g., "开始探讨需求" or similar phrases), you MUST immediately engage in requirement discussion
- **No Code Execution**: During requirement discussion phase, DO NOT execute any code, terminal commands, or implementation tasks
- **Focus on Understanding**: Concentrate entirely on understanding the user's needs, clarifying ambiguities, and exploring possibilities
- **Ask Clarifying Questions**: Use the `ask_followup_question` tool to gather specific information about requirements
- **Document Discussion**: Take notes on key points discussed and decisions made during the requirement discussion
- **Transition to Implementation**: Only after requirements are fully understood and confirmed by the user should you proceed to create implementation plans

## Implementation Plan Documentation Management

**IMPORTANT**: All implementation plans must be stored in a structured folder system with specific naming and organization rules:

### Documentation Storage Structure
Create a folder named `implementation-plans/` in the project root with the following structure:
```
implementation-plans/
├── pending/           # Pending task documents
├── completed/         # Completed task documents
└── design/            # Design documents
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

#### For Design Documents
1. **Create Design Document**: When creating design documents, place them in `implementation-plans/design/`
2. **Name Design Document**: Use the naming convention: `YYYY-MM-DD-HHMM-DesignName.md`
3. **Write Content**: Write the complete design document in Chinese
4. **Update Design**: Keep design documents updated as the design evolves
5. **Archive Design**: Design documents remain in the `design/` folder for reference

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

## Project Architecture Rules (Non-Obvious Only)

### Core Technology Stack
- **Frontend**: React 19 + TypeScript 5.8 + Vite 7 + React Router 7
- **Backend**: Tauri 2.7 + Rust with custom crates
- **State Management**: Jotai (atomic state) + Zustand (complex state)
- **UI Framework**: Radix UI Themes + Primitives + Headless UI
- **Styling**: TailwindCSS 3.4 + CSS Custom Properties + Tailwind Variants
- **Database**: SQLite + Diesel ORM with migrations

### Plugin System Constraints
- All music providers MUST implement `AudioProvider` trait - no direct integration allowed
- Plugins are stateless by design - hidden caching layer assumes this pattern
- Plugin communication uses event system, not direct function calls - enforces loose coupling
- Plugin capabilities are declared, not discovered - enables static validation
- Plugin UUIDs are primary identifiers, not names - prevents naming conflicts
- Multi-provider support: Spotify, YouTube, Bilibili with unified interface
- Provider registry with dynamic loading and authentication management

### Type System Architecture
- Types shared between Rust and TypeScript via `crates/types` - no duplication allowed
- SDK types (plugin) and app types (internal) are separate - adapter pattern required
- TypeScript bindings auto-generated via ts-rs - manual editing will be overwritten
- Database models in `types` crate with `db` feature - separates concerns
- Frontend cannot use SDK types directly - must go through adapter conversion
- Full type safety across Rust/TS boundary with runtime validation

### Data Flow Constraints
- Frontend → Tauri Commands → Music Router → Plugin Manager → Plugins
- Stream URLs resolved on-demand, not stored - reduces storage and handles expiration
- Search results from multiple providers are merged in adapter - not in frontend
- Plugin state persisted in database, not in-memory - survives restarts
- Configuration loaded selectively by key path - not entire config at once
- Type conversion happens at multiple layers - adds overhead but necessary

### Performance Bottlenecks
- Plugin operations have 5-second timeout - blocking operations will fail
- Type conversion happens at multiple layers - adds overhead but necessary
- Database access through Diesel ORM - adds abstraction cost
- Plugin loading happens at startup - affects application launch time
- Stream URL resolution is synchronous from user perspective - async internally
- Heavy operations delegated to Rust backend for performance

### Hidden Coupling Dependencies
- `crates/types` is circular dependency with multiple crates - intentional design
- Plugin SDK depends on core types - creates tight coupling for type safety
- Database migrations require specific ordering - implicit dependencies
- Frontend path aliases don't match actual directory structure - build-time resolution
- Audio player plugin has hidden dependencies on system audio APIs
- Theme system has complex dependencies on CSS custom properties and gradient generation

### Architectural Decisions
- Monorepo with multiple crates instead of single binary - enables plugin isolation
- SQLite with Diesel instead of NoSQL - relational model for music metadata
- Custom plugin system instead of existing solutions - music-specific requirements
- Tauri instead of Electron - better performance for audio processing
- Custom audio player instead of system default - cross-platform consistency
- React 19 with concurrent features for improved UI responsiveness
- Advanced theming system with gradient backgrounds and dynamic CSS variables

### Extension Points
- New music providers must implement `AudioProvider` trait in separate crate
- New Tauri plugins go in `lib/` directory - not standard location
- New database migrations require date-prefixed directories - versioning scheme
- New shared types must be in `crates/types` with proper serde attributes
- New frontend components must use existing path aliases - consistency requirement
- Theme extensions must use CSS custom properties and gradient system
- Internationalization additions must follow alphabetical sorting in namespace files

### Testing Architecture
- Each crate has its own tests - no integration test directory
- Plugin tests require full plugin system - complex setup
- Type generation tests require specific feature flags - not default
- Database tests use test migrations - separate from production
- Frontend tests use Vite environment - not standard Jest
- Component tests with React Testing Library for UI components
- Integration tests for critical user journeys

### Advanced Features Architecture
- **Provider System**: Multi-provider with registry, authentication, API abstraction
- **File Scanning**: Cross-platform with Android support, metadata extraction, efficient storage
- **Theme Engine**: Dynamic backgrounds, color management, CSS generation, user customization
- **Layout System**: Flexible sidebar + content + player areas with multiple toolbar modes
- **Internationalization**: Multi-language support with fallback strategies and namespace organization

### File Organization Patterns
```
src/
├── components/           # React components
│   ├── ui/              # Reusable UI primitives
│   ├── layout/          # Layout components (desktop/mobile)
│   ├── modules/         # Feature-specific modules
│   └── common/          # Shared components
├── atoms/               # Jotai state atoms
├── lib/                 # Utility libraries
├── hooks/               # Custom React hooks
├── providers/           # Context providers
├── services/            # API services
├── types/               # TypeScript definitions
└── styles/              # CSS files

crates/
├── database/            # Diesel ORM + migrations
├── file_scanner/        # Local music file scanning
├── providers/           # Music provider integrations
├── types/               # Shared type definitions
├── themes/              # Theme system
└── settings/            # Settings management

### Rust Code Generation Guidelines
When designing architecture that involves Rust code generation, refer to the Rust Code Generation Rules in the Code mode documentation. These rules cover:
- Ownership and borrowing principles
- Memory safety patterns
- Async programming patterns
- Testing Rust code

All architectural decisions involving Rust code must comply with these rules to ensure code quality and maintainability.
```