/**
 * Copyright (c) Meta Platforms, Inc. and affiliates.
 *
 * This source code is licensed under both the MIT license found in the
 * LICENSE-MIT file in the root directory of this source tree and the Apache
 * License, Version 2.0 found in the LICENSE-APACHE file in the root directory
 * of this source tree.
 */

/**
  * The sidebars for buck2 documentation work slightly differently than normal.
  * Normally when globbing you don't have control over any ordering (in an easy to manage way)
  * Instead, we do some processing on the manualSidebar array to remove any manually specified
  * files from the autogenerated glob, and keep the manuallly specified ones in order.
  *
  * - To specify manual ordering, just put the filename into the list of items.
  * - New sections should be in a subdirectory, and should generally end have an autogenerated
  *   item as their last item.
  * - Directories that should be excluded from sidebars should be added to the
  *   'universallyExcludedDirs' set below
  *
  * If you're curious how this works, look at `generateSidebarExclusions` and
  * `filterItems` in this module, and `sidebarItemsGenerator` in docusaurus.config.js. Note
  * that `sidebarItemsGenerator` runs for each "autogenerated" item, so that's why we
  * keep track of all globs that have been specified. We need to make sure that only things
  * in "developers/" are included in the developers glob, e.g.
  */

import { isInternal } from "docusaurus-plugin-internaldocs-fb/internal";
import type { SidebarsConfig } from '@docusaurus/plugin-content-docs';


export const sidebars: SidebarsConfig = {
  main: [
    'index',
    {
      type: 'category' as const,
      label: 'About Buck2',
      items: [
        'about/why',
        // The getting_started page is for OSS only.
        isInternal() ? null : 'about/getting_started',
        {
          type: 'category' as const,
          label: 'Benefits',
          items: [
            'about/benefits/compared_to_buck1',
          ],
        },
        isInternal() ? 'about/knowledge_sharing' : null,
        isInternal() ? null : 'about/bootstrapping',
      ].flatMap(x => x !== null ? [x] : []),
    },
    {
      type: 'category',
      label: 'Concepts',
      items: [
        'concepts/key_concepts',
        'concepts/concept_map',
        'concepts/build_rule',
        'concepts/build_file',
        'concepts/build_target',
        'concepts/target_pattern',
        'concepts/buck_out',
        'concepts/visibility',
        'concepts/daemon',
        'concepts/labels',
        'concepts/isolation_dir',
        'concepts/buckconfig',
        'concepts/configurations',
        'concepts/modifiers',
        'concepts/glossary',
      ],
    },
    {
      type: 'category' as const,
      label: 'Buck2 Users',
      items: [
        isInternal() ? 'users/migration_guide' : null,
        {
          type: 'category' as const,
          label: 'Commands',
          items: [
            { type: 'autogenerated' as const, dirName: 'users/commands' },
          ],
        },
        {
          type: 'category' as const,
          label: 'How-tos',
          items: [
            'users/how_tos/modifiers_setup',
            'users/how_tos/modifiers_package',
            'users/how_tos/modifiers_target',
            'users/how_tos/modifiers_cli',
            'users/how_tos/compilation_database',
          ],
        },
        'users/cheat_sheet',
        {
          type: 'category' as const,
          label: 'Troubleshooting',
          items: [
            isInternal() ? 'users/faq/getting_help' : null,
            'users/faq/common_issues',
            isInternal() ? 'users/faq/meta_issues' : null,
            isInternal() ? 'users/faq/meta_installation' : null,
            isInternal() ? 'users/faq/remote_execution' : null,
            'users/faq/starlark_peak_mem',
            'users/faq/buck_hanging',
            isInternal() ? 'users/faq/how_to_bisect' : null,
            isInternal() ? 'users/faq/how_to_expedite_fix' : null,
          ].flatMap(x => x !== null ? [x] : []),
        },
        {
          type: 'category' as const,
          label: 'Build Observability',
          items: [
            'users/build_observability/interactive_console',
            'users/build_observability/logging',
            'users/build_observability/build_report',
            isInternal() ? 'users/build_observability/observability' : null,
            isInternal() ? 'users/build_observability/scuba' : null,
            isInternal() ? 'users/build_observability/ods' : null,
          ].flatMap(x => x !== null ? [x] : []),
        },
        'users/remote_execution',
        {
          type: 'category' as const,
          label: 'Queries',
          items: [
            { type: 'autogenerated' as const, dirName: 'users/query' },
          ],
        },
        {
          type: 'category' as const,
          label: 'Advanced Features',
          items: [
            'users/advanced/deferred_materialization',
            'users/advanced/restarter',
            'users/advanced/in_memory_cache',
            'users/advanced/external_cells',
            isInternal() ? 'users/advanced/offline_build_archives' : null,
            isInternal() ? 'users/advanced/vpnless' : null,
          ].flatMap(x => x !== null ? [x] : []),
        },
      ].flatMap(x => x !== null ? [x] : []),
    },
    {
      type: 'category',
      label: 'Rule Authors',
      items: [
        'rule_authors/writing_rules',
        'rule_authors/transitive_sets',
        'rule_authors/configurations',
        'rule_authors/configurations_by_example',
        'rule_authors/configuration_transitions',
        'rule_authors/dynamic_dependencies',
        'rule_authors/anon_targets',
        'rule_authors/test_execution',
        'rule_authors/optimization',
        isInternal() ? 'rule_authors/rule_writing_tips' : null,
        'rule_authors/incremental_actions',
        'rule_authors/alias',
        'rule_authors/local_resources',
        'rule_authors/package_files',
        isInternal() ? 'rule_authors/client_metadata' : null,
        isInternal() ? 'rule_authors/action_error_handler' : null,
        'rule_authors/dep_files',
      ].flatMap(x => x !== null ? [x] : []),
    },
    {
      type: 'category',
      label: 'BXL Developers',
      items: [
        {
          type: 'category',
          label: 'About BXL',
          items: [
            'bxl/index',
            isInternal() ? 'bxl/testimonials' : null,
          ].flatMap(x => x !== null ? [x] : []),
        },
        'bxl/tutorial',
        {
          type: 'category',
          label: 'How-to guides',
          items: [
            'bxl/how_tos/basic_how_tos',
            'bxl/how_tos/how_to_cache_and_share_operations',
            'bxl/how_tos/how_to_handle_errors',
            'bxl/how_tos/how_to_catch_building_artifacts_errors',
            'bxl/how_tos/how_to_run_actions_based_on_the_content_of_artifact',
            'bxl/how_tos/how_to_use_target_universe',
            'bxl/how_tos/how_to_collect_telemetry_events'
          ]
        },
        {
          type: 'category',
          label: 'Explanation',
          items: [
            'bxl/explanation/basics',
            'bxl/explanation/labels_and_nodes',
            'bxl/explanation/bxl_cquery_vs_cli_cquery',
          ]
        },
        'bxl/faq',
        {
          type: 'ref',
          label: 'BXL APIs',
          id: 'api/bxl/index',
        },
      ],
    },
    {
      type: 'category',
      label: 'Buck2 Developers',
      items: [
        {
          type: 'category' as const,
          label: 'Architecture',
          items: [
            'developers/architecture/buck2',
            'developers/architecture/buck1_vs_buck2',
          ],
        },
        isInternal() ? 'developers/developers' : null,
        isInternal() ? 'developers/heap_profiling' : null,
        'developers/what-ran',
        {
          type: 'category' as const,
          label: 'Starlark Language',
          items: [
            { type: 'autogenerated' as const, dirName: 'developers/starlark' },
          ],
        },
        'developers/request_for_comments',
        'developers/windows_cheat_sheet',
      ].flatMap(x => x !== null ? [x] : []),
    },
    {
      type: 'category',
      label: 'Insights and Knowledge',
      items: [
        'insights_and_knowledge/modern_dice',
      ],
    }
  ],
  apiSidebar: [
    {
      type: 'doc',
      id: 'api',
    },
    {
      type: 'doc',
      id: 'prelude/globals',
      label: 'Rules',
    },
    {
      type: 'category',
      label: 'Starlark APIs',
      items: [{ type: 'autogenerated', dirName: 'api/starlark' }],
      link: { type: 'doc', id: 'api/starlark/index' },
    },
    {
      type: 'category',
      label: 'Build APIs',
      items: [{ type: 'autogenerated', dirName: 'api/build' }],
      link: { type: 'doc', id: 'api/build/index' },
    },
    {
      type: 'category',
      label: 'BXL APIs',
      items: [
        { type: 'autogenerated', dirName: 'api/bxl' },
      ],
      link: { type: 'doc', id: 'api/bxl/index' },
    },
    {
      type: 'category',
      label: 'BXL utils functions',
      items: [
        { type: 'autogenerated', dirName: 'api/bxl_utils' },
      ],
    }
  ],
};

export function postProcessItems(items) {
  // First, handle recursive categories
  const result = items.map((item) => {
    if (item.type === 'category') {
      return { ...item, items: postProcessItems(item.items) };
    }
    return item;
  });

  // Filter out index pages. Docusaurus only does this correctly on subcategories
  return result.filter((item) => {
    if (item.type === 'doc' && item.id) {
      return item.id.split("/").at(-1) !== "index";
    } else {
      return true;
    }
  });
}
