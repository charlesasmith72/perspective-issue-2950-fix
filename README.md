# Perspective — Custom Plugin HTML Export Fix

## Overview

This project fixes FINOS Perspective Issue #2950, where exporting a viewer with a custom plugin generates HTML that cannot reload that plugin correctly.

The original export logic only stored the plugin name and assumed every plugin could be loaded as an official Perspective package. That works for built-in plugins, but fails for custom plugins because Perspective does not know the module path that defined them.

This change adds an optional module argument to `registerPlugin()`:

```javascript
await customElements
    .get("perspective-viewer")
    .registerPlugin(
        "perspective-viewer-summary",
        "./perspective-viewer-summary.js"
    );
```

Perspective now stores both the plugin name and its module location. During HTML export, custom modules are imported and re-registered before the saved viewer layout is restored.

The project also includes:

* A broken example that reproduces the original issue.
* A fixed example that uses the new module-aware registration.
* A custom Summary renderer for testing.
* Build and test instructions for comparing both behaviors.


**Issue #2950 — Registering a custom plugin breaks HTML export**

[https://github.com/perspective-dev/perspective/issues/2950](https://github.com/perspective-dev/perspective/issues/2950)

The issue reports that when a Perspective viewer uses a custom visualization plugin and the viewer is exported as HTML, the generated file incorrectly assumes that the custom plugin is an official Perspective package available from the Perspective CDN namespace.

Because the custom package does not exist at that generated CDN location, the exported viewer cannot load the plugin and fails during restoration.

## Problem

Perspective allows visualization plugins to be registered with the viewer:

```javascript
await customElements
    .get("perspective-viewer")
    .registerPlugin("perspective-viewer-summary");
```

Before this change, Perspective stored only the plugin name.

During HTML export, Perspective attempted to convert every registered plugin name into an official package URL.

For example:

```text
perspective-viewer-summary
```

was treated like an official package:

```text
@finos/perspective-viewer-summary
```

The exported HTML then attempted to load the plugin from a generated CDN URL.

This works for official Perspective plugins such as:

```text
perspective-viewer-datagrid
perspective-viewer-d3fc
```

It does not work for arbitrary custom plugins because Perspective does not know where those modules are located.

As a result:

* The custom plugin module is not loaded.
* The generated CDN request fails.
* The plugin is unavailable when the saved layout is restored.
* The exported HTML cannot recreate the original viewer.

## Solution

The plugin registration API now accepts an optional module location.

Existing registrations remain valid:

```javascript
await customElements
    .get("perspective-viewer")
    .registerPlugin("perspective-viewer-datagrid");
```

Custom plugins can now provide their module location:

```javascript
await customElements
    .get("perspective-viewer")
    .registerPlugin(
        "perspective-viewer-summary",
        "./perspective-viewer-summary.js"
    );
```

Perspective stores the optional module together with the plugin name.

When HTML is exported:

* Official plugins continue using Perspective’s existing CDN package resolution.
* Custom plugins use the module location supplied during registration.
* The generated HTML imports the custom module.
* The generated HTML re-registers the custom plugin.
* The viewer layout is restored only after the plugin is available.

## API Change

The TypeScript plugin registration API changed from:

```typescript
registerPlugin(name: string): Promise<void>;
```

to:

```typescript
registerPlugin(name: string, module?: string): Promise<void>;
```

The second argument is optional, preserving backward compatibility with existing Perspective applications.

## Example Usage

### Existing official plugin

```javascript
await customElements
    .get("perspective-viewer")
    .registerPlugin("perspective-viewer-datagrid");
```

Perspective continues resolving this through its normal package convention.

### Custom local plugin

```javascript
await customElements
    .get("perspective-viewer")
    .registerPlugin(
        "perspective-viewer-summary",
        "./perspective-viewer-summary.js"
    );
```

### Custom remote plugin

```javascript
await customElements
    .get("perspective-viewer")
    .registerPlugin(
        "perspective-viewer-example",
        "https://example.com/perspective-viewer-example.js"
    );
```

The supplied module must remain accessible from the location where the exported HTML is opened.

## Implementation

The implementation updates the complete plugin export path across the TypeScript and Rust/WASM portions of Perspective.

### TypeScript API

The public viewer API accepts an optional module location:

```typescript
registerPlugin(name: string, module?: string): Promise<void>;
```

### Rust/WASM Interface

The Rust-facing registration method receives the optional module value and passes it to the plugin registry.

### Plugin Registry

Each plugin record now contains:

```rust
pub struct PluginRecord {
    pub name: String,
    pub module: Option<String>,
}
```

This allows Perspective to distinguish between:

* A normal plugin registration that relies on package-name resolution.
* A custom plugin registration that provides an explicit module location.

### HTML Export Metadata

The export pipeline classifies registered plugins as either:

```rust
ExportPlugin::Package
```

or:

```rust
ExportPlugin::Module
```

Official Perspective plugins continue to use package-based CDN resolution.

Custom plugins use their supplied module location.

### Generated HTML

The generated export performs the following sequence:

1. Imports Perspective.
2. Imports official Perspective plugins.
3. Imports custom plugin modules.
4. Replays custom plugin registration.
5. Creates the exported table.
6. Loads the worker into the viewer.
7. Restores the saved viewer layout.

The custom plugin must be registered before the layout is restored because the layout references the plugin by name.

## Main Files Changed

```text
rust/perspective-viewer/src/ts/extensions.ts
rust/perspective-viewer/src/rust/lib.rs
rust/perspective-viewer/src/rust/renderer/registry.rs
rust/perspective-viewer/src/rust/tasks/copy_export.rs
rust/perspective-viewer/src/rust/queries/export_app.rs
```

## Demonstration Files

The demonstration is located at:

```text
examples/blocks/src/custom-plugin-export/
```

It contains:

```text
index.html
broken.html
perspective-viewer-summary.js
```

### `broken.html`

This page demonstrates the original behavior described in Issue #2950.

The custom plugin is registered without an explicit module location.

This allows the reviewer to observe why the original HTML export process cannot reliably locate an arbitrary custom plugin.

### `index.html`

This page demonstrates the corrected implementation.

The custom Summary plugin is registered with:

```javascript
await customElements
    .get("perspective-viewer")
    .registerPlugin(
        "perspective-viewer-summary",
        "./perspective-viewer-summary.js"
    );
```

The exported HTML can then import the correct module and restore the custom renderer.

### `perspective-viewer-summary.js`

This file implements the custom Summary renderer used by both demonstrations.

The plugin provides a small summary-style visualization so the custom renderer is visibly different from the built-in Perspective plugins.

# Installation

## 1. Clone the Repository

```bash
git clone --branch issue-2950-recovery https://github.com/charlesasmith72/perspective-issue-2950-fix.git
```

Enter the repository:

```bash
cd perspective-issue-2950-fix
```

## 2. Install Dependencies

```bash
pnpm install
```

## 3. Configure the Perspective Workspace

Run:

```bash
pnpm run setup
```

When the package-selection screen appears, select:

```text
@perspective-dev/viewer
```

Use the keyboard controls shown by the setup tool:

```text
Arrow keys — move through the list
Space — select or clear an item
Enter — continue
```

For the remaining prompts, use:

```text
Run debug build? No
Use docker for build env? No
```

Pressing Enter accepts `No` when the prompt displays:

```text
(y/N)
```

## 4. Build Perspective

Run:

```bash
pnpm run build
```

The relevant parts of this exercise are built when the output includes packages such as:

```text
@perspective-dev/client
@perspective-dev/viewer
@perspective-dev/viewer-charts
@perspective-dev/viewer-datagrid
```

The viewer build should generate files such as:

```text
rust/perspective-viewer/dist/esm/perspective-viewer.js
rust/perspective-viewer/dist/cdn/perspective-viewer.js
rust/perspective-viewer/dist/wasm/perspective-viewer.wasm
```

## Documentation Build Note

The root Perspective build may continue into unrelated packages after successfully building the viewer.

The documentation package runs:

```text
docker compose run --rm mdbook build
```

On a machine where Docker is unavailable or the current user does not have Docker permission, the command may eventually end with:

```text
docker: Permission denied
```

That error occurs in the separate documentation package after the Perspective viewer, client, WASM, and demonstration dependencies have already compiled.

It is not caused by the custom plugin export implementation.

# Running the Demonstration

Start the Perspective Blocks development server:

```bash
pnpm run start blocks
```

Keep that terminal open while testing.

Open:

```text
http://localhost:8080/custom-plugin-export/
```

# Testing the Original Broken Behavior

Open:

```text
http://localhost:8080/custom-plugin-export/broken.html
```

The broken demonstration registers the custom plugin using only its custom-element name:

```javascript
await customElements
    .get("perspective-viewer")
    .registerPlugin("perspective-viewer-summary");
```

Perspective knows the plugin name, but it does not know the module that created the plugin.

When HTML export attempts to recreate the viewer, the original export logic can only guess where that plugin should be loaded from.

The plugin name:

```text
perspective-viewer-summary
```

is treated like an official Perspective package:

```text
@finos/perspective-viewer-summary
```

That package does not exist in the official Perspective package namespace.

## Broken Version Test Steps

1. Open the browser developer tools.
2. Open the Console tab.
3. Confirm that the custom Summary plugin is visible in the original page.
4. Use the viewer’s export controls to export the page as HTML.
5. Open the generated HTML.
6. Inspect the browser console and network activity.
7. Inspect the generated HTML source.

The exported page may report:

* A failed CDN request.
* A CORS error.
* A MIME-type error.
* A module-loading error.
* An unavailable plugin during layout restoration.

The exact browser message may vary.

The underlying problem is that the export does not know where the custom plugin module is located.

## What to Look for in the Broken Export

Open the generated HTML in a text editor and search for:

```text
perspective-viewer-summary
```

The export should show that the plugin is being treated as a package instead of using the local module:

```text
./perspective-viewer-summary.js
```

This demonstrates the original Issue #2950 behavior.

# Testing the Fixed Behavior

Open:

```text
http://localhost:8080/custom-plugin-export/
```

The fixed demonstration registers the plugin with its module location:

```javascript
await customElements
    .get("perspective-viewer")
    .registerPlugin(
        "perspective-viewer-summary",
        "./perspective-viewer-summary.js"
    );
```

## Fixed Version Test Steps

1. Open the browser developer tools.
2. Open the Console tab.
3. Confirm that the Perspective viewer loads.
4. Confirm that the custom Summary renderer is available.
5. Confirm that the custom Summary renderer displays data.
6. Use the viewer’s export controls to export the viewer as HTML.
7. Open the generated HTML.
8. Confirm that the exported page restores the custom Summary renderer.
9. Inspect the generated HTML source.

The exported HTML should:

* Import the custom module from the supplied location.
* Register `perspective-viewer-summary`.
* Load the exported table.
* Load the Perspective worker into the viewer.
* Restore the saved layout.
* Display the custom Summary renderer.

## What to Look for in the Fixed Export

Open the generated HTML in a text editor and search for:

```text
perspective-viewer-summary.js
```

The generated export should contain an import for the custom module rather than an invented official Perspective package URL.

Also search for:

```text
registerPlugin
```

The generated code should replay the plugin registration before restoring the viewer layout.

# Expected Difference

## Broken Version

The plugin is registered without a module:

```javascript
registerPlugin("perspective-viewer-summary");
```

Perspective records only:

```text
Plugin name:
perspective-viewer-summary

Module:
unknown
```

During export, Perspective assumes that the plugin is an official package.

The generated HTML cannot correctly locate the custom module.

## Fixed Version

The plugin is registered with a module:

```javascript
registerPlugin(
    "perspective-viewer-summary",
    "./perspective-viewer-summary.js"
);
```

Perspective records:

```text
Plugin name:
perspective-viewer-summary

Module:
./perspective-viewer-summary.js
```

During export, Perspective uses the explicit module location.

The generated HTML imports and re-registers the plugin before restoring the viewer.

# Validation Performed

The implementation was tested from a separate clean clone of the repository.

The clean-clone test confirmed that:

* Project dependencies installed.
* Perspective setup completed.
* The Perspective JavaScript client compiled.
* The Perspective viewer Rust code compiled.
* The Perspective viewer WASM bundle compiled.
* The Perspective ESM bundle was generated.
* The Perspective CDN bundle was generated.
* The viewer charts package compiled.
* The viewer datagrid package compiled.
* The custom Summary plugin loaded in the demonstration.
* The fixed export imported the supplied custom module.
* The fixed export replayed custom plugin registration.
* The fixed export restored the custom renderer.
* Official Perspective plugins continued using the normal package-resolution path.

# Backward Compatibility

The change preserves the original one-argument API:

```javascript
registerPlugin("perspective-viewer-datagrid");
```

Existing applications do not need to provide a module.

The optional module is only needed when the HTML export process cannot determine a plugin’s source from the normal Perspective package convention.

This keeps existing behavior intact while adding explicit support for custom plugin exports.

# Design Decisions

## Optional Module Argument

The module argument is optional so existing applications remain compatible.

A breaking API change was not necessary to solve the issue.

## Capture the Module During Registration

Once a custom element is registered, the browser does not provide a reliable standard mechanism for determining the JavaScript module URL that originally defined it.

The most deterministic solution is to capture the module location when the plugin is registered.

## Preserve Official Package Resolution

Official Perspective plugins already have a known package naming convention.

The implementation leaves that path unchanged.

Only registrations that provide an explicit module use the new custom-module export path.

## Register Before Restoring

The exported layout contains the selected plugin name.

Calling:

```javascript
viewer.restore(layout);
```

before registering the custom plugin would cause restoration to reference a renderer that is not yet available.

The generated HTML therefore imports and registers custom plugins first.

## Avoid a Hosted Plugin Registry

Issue #2950 mentions a possible community plugin registry as a longer-term direction.

A registry would introduce hosting, governance, package validation, versioning, and security concerns that are outside the immediate bug.

The optional module argument fixes the current export problem without requiring new infrastructure.

# Current Scope

This implementation supports module locations that can be used by JavaScript `import`, including:

```text
./perspective-viewer-summary.js
../plugins/custom-renderer.js
https://example.com/plugins/custom-renderer.js
```

The module must remain accessible when the exported HTML is opened.

For example, an export that references:

```text
./perspective-viewer-summary.js
```

must be kept in a location where that relative file path remains valid.

This implementation does not copy arbitrary third-party plugin source code directly into the exported HTML.

# Possible Future Improvements

Potential follow-up work includes:

* Embedding custom plugins directly into fully self-contained exports.
* Adding automated browser tests for custom plugin HTML export.
* Validating custom module locations during registration.
* Supporting import maps.
* Supporting structured plugin metadata.
* Adding clearer export-time diagnostics for missing modules.
* Adding version metadata for custom plugins.
* Supporting integrity hashes for remote plugin modules.
* Creating an optional community plugin registry.
* Allowing applications to define custom export resolution policies.

# Technical Exercise Summary

This submission addresses a real open limitation inside the FINOS Perspective codebase.

The work required tracing behavior through:

* The public TypeScript viewer API.
* The Rust/WASM boundary.
* The internal plugin registry.
* Plugin metadata storage.
* HTML export metadata collection.
* Generated JavaScript module imports.
* Worker and table initialization.
* Viewer layout restoration.
* Browser testing with a custom renderer.
* Backward compatibility with official Perspective plugins.

The final implementation preserves existing plugin behavior while giving custom plugins an explicit and deterministic way to remain functional after HTML export.


## Original Perspective Project

[View the original Perspective README](./PERSPECTIVE_README.md)
