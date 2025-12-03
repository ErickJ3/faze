# Faze UI

## Completed

### Infrastructure & Setup
- [x] TypeScript types for all API models (trace, log, metric, service, common)
- [x] Axios API client configured with base URL (localhost:7070/api)
- [x] API services for all resources (traces, logs, metrics, services)
- [x] React Query setup with QueryClient
- [x] Custom hooks for data fetching (useTraces, useLogs, useMetrics, useServices)
- [x] Formatter utilities (time, duration, number)
- [x] Constants and color mappings

### UI Components & Shared
- [x] Toast notification system with provider
- [x] Copy button with toast feedback
- [x] Loading state component
- [x] Error state component
- [x] Duration badge component
- [x] Attributes viewer (expandable arrays)
- [x] Pagination component (reusable)
- [x] Command palette with Ctrl+K (vim-like navigation: h,j,k,l)
- [x] Theme toggle (dark/light mode)
- [x] Theme persistence with localStorage
- [x] Dark mode as default theme

### Features - Services
- [x] Services list page
- [x] Navigation to service-specific traces
- [x] Service detail page with:
  - Service overview (traces, logs, metrics count)
  - Health status indicator (Healthy/Degraded/Unhealthy based on error rate)
  - Performance metrics (avg duration, p95, error rate)
  - Response time bar chart (last 20 traces, green/red color coding)
  - Recent traces for service (last 10) with navigation
  - Recent logs for service (last 10) with severity badges
  - Service-specific metrics grid with latest values
- [x] Fixed metric value display (data_points[].value structure)
- [x] Defensive null checks for duration_ms and severity_level

### Features - Traces
- [x] Traces list page with filters
- [x] Service filter with nuqs (URL state management)
- [x] Duration filters (min/max) with nuqs
- [x] Search/filter by trace ID, span name, and service name
- [x] Pagination with configurable page size (10, 25, 50, 100)
- [x] Traces table with enriched data:
  - Root span operation with kind badge (server, client, producer, consumer, internal)
  - Root span name
  - Service name
  - Duration with color-coded badge
  - Span count
  - Status (OK/ERROR)
  - Relative time
  - Trace ID (truncated)
- [x] Trace detail page
- [x] Span timeline visualization (Sentry-style)
  - Hierarchical tree with depth-based positioning
  - Inline Gantt chart bars with percentage markers
  - Green bars for normal spans, red for errors
  - Error handling for invalid data
  - Sorting by start time
  - Empty parent_span_id handling
  - Hover tooltips with span names and durations
- [x] Span detail sheet (side panel, 600px width)
  - Tabs for Attributes and JSON view
  - Copy buttons for IDs
  - Proper background colors
- [x] React Query with placeholderData: keepPreviousData to prevent re-renders

### Features - Logs
- [x] Logs list page
- [x] Service and severity level filters with nuqs
- [x] Search/filter by log body and service name
- [x] Pagination with configurable page size (10, 25, 50, 100)
- [x] Severity badges with color coding
- [x] Log detail view with attributes
- [x] React Query with placeholderData: keepPreviousData

### Features - Metrics
- [x] Metrics grid view
- [x] Service filter with nuqs
- [x] Metric type display
- [x] Trend indicators (up/down/stable)
- [x] Mini sparkline charts (last 10 data points)
- [x] Metric cards with value from latest data point
- [x] React Query with placeholderData: keepPreviousData
- [x] Enhanced metric cards on service detail page with SVG sparklines

### Features - Dashboard
- [x] Stats cards (total traces, logs, metrics)
- [x] Recent traces table
- [x] Service navigation

### Features - Settings
- [x] Auto-refresh configuration (5-300 seconds)
- [x] Fixed input to allow deletion with onFocus select
- [x] Min/max validation

### CSS & Styling
- [x] Tailwind CSS v4 setup
- [x] Dark theme with proper oklch colors
- [x] Light theme with proper oklch colors (timeline visibility fixed)
- [x] Custom scrollbar styling
- [x] Mono/minimalist design (black & white)
- [x] Zero border radius (--radius: 0rem)
- [x] Geist Mono font for monospace
- [x] Green color scheme for normal spans/traces (Sentry-style)
- [x] Red color scheme for error spans/traces

### Performance & Optimization
- [x] nuqs for URL state management (prevents unnecessary re-renders)
- [x] React Query placeholderData for smooth filter transitions
- [x] Proper error boundaries
- [x] Loading states

## In Progress

### Backend Integration
- [x] Verified backend API endpoints are working
- [x] Backend returns root_span_name and root_span_kind in TraceInfo response
- [x] Tested with real OTLP data from faze-collector
- [x] All API endpoints match frontend expectations

### Features - Traces
- [ ] Add date range picker for time filtering
- [ ] Export traces (JSON format)
- [ ] Trace comparison view (compare 2+ traces side-by-side)

### Features - Logs
- [ ] Date range picker for time filtering
- [ ] Export logs (JSON/CSV format)
- [ ] Log aggregation view (group by service, severity, time)

### Features - Metrics
- [ ] Metric detail page with historical data
- [ ] Date range picker for time filtering
- [ ] Metric comparison view
- [ ] Export metrics (JSON/CSV format)

### Features - Dashboard
- [ ] Add charts (traces over time, errors over time)
- [ ] Add service health overview
- [ ] Add alerting/notification system
- [ ] Add recent errors section
- [ ] Customizable dashboard layout

### Features - Services
- [ ] Service topology view (service dependencies graph)
- [ ] Enhanced performance metrics (p50, p99 latencies)

### Quality of Life
- [ ] Keyboard shortcuts reference modal (show with ?)
- [ ] Add breadcrumbs for navigation
- [ ] Add "Recently viewed" section
- [ ] Add favorites/bookmarks for traces/services
- [ ] Add URL sharing (copy URL with current filters)
- [ ] Add table column visibility toggle
- [ ] Add table column sorting
- [ ] Add table row density options (compact/normal/comfortable)

### Performance
- [ ] Implement virtual scrolling for large tables
- [ ] Add request caching strategies
- [ ] Add optimistic updates where applicable
- [ ] Implement request debouncing for filters

### Testing
- [ ] Unit tests for formatters
- [ ] Unit tests for API services
- [ ] Component tests for major features
- [ ] E2E tests for critical flows

### Documentation
- [ ] Component documentation
- [ ] API integration guide
- [ ] Development setup guide
- [ ] Deployment guide

## Enhancements

### Advanced Features
- [ ] Trace sampling configuration
- [ ] Custom alerts and rules
- [ ] Anomaly detection
- [ ] Service SLA monitoring
- [ ] Custom dashboards builder
- [ ] Team collaboration features (comments, annotations)
- [ ] Integration with external services (Slack, PagerDuty)
- [ ] Multi-tenancy support
- [ ] RBAC (Role-Based Access Control)

### UI/UX
- [ ] Add more theme options (custom color schemes)
- [ ] Add accessibility improvements (ARIA labels, keyboard navigation)
- [ ] Add mobile responsive design
- [ ] Add onboarding tour for new users
- [ ] Add context help tooltips

### Data Visualization
- [ ] Flame graphs for performance analysis
- [ ] Service maps with real-time data
- [ ] Heatmaps for request patterns
- [ ] Custom query builder
- [ ] SQL-like query language for advanced filtering
