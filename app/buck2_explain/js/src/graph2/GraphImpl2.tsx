/**
 * Copyright (c) Meta Platforms, Inc. and affiliates.
 *
 * This source code is licensed under both the MIT license found in the
 * LICENSE-MIT file in the root directory of this source tree and the Apache
 * License, Version 2.0 found in the LICENSE-APACHE file in the root directory
 * of this source tree.
 */

import React, {useRef, useState} from 'react'
import {Build} from '../fbs/explain'
import {RuleTypeDropdown} from './RuleTypeDropdown'
import {Node} from './GraphView2'
import {GraphViz2} from './GraphViz2'
import {LinkObject, NodeObject} from 'react-force-graph-2d'
import {formatTargetLabel} from '../formatTargetLabel'

enum DisplayType {
  rootNode,
  passesFilters,
  changedFiles,
  hidden,
  highlighted,
  actionsRan,
}

const displayTypeColors: {[key in DisplayType]: string} = {
  // https://coolors.co/fb5012-9c528b-00c49a-e6d3a3-2274a5
  [DisplayType.rootNode]: '#1a181b',
  [DisplayType.passesFilters]: '#1c77c3',
  [DisplayType.changedFiles]: '#00C49A',
  [DisplayType.highlighted]: '#9C528B',
  [DisplayType.actionsRan]: '#2274A5',
  [DisplayType.hidden]: 'gray', // doesn't matter
}

// More than this and graph starts to feel sluggish
const MAX_NODES = 500

interface DisplayNode extends Node {
  allowedDeps: Map<number, number>
  displayType: DisplayType
}

function showNode(node: DisplayNode) {
  return node.displayType != DisplayType.hidden
}

// Here it goes everything that has to recompute on user interaction.
// On big graphs recomputing less matters
export function GraphImpl2(props: {
  nodes: Map<number, Node>
  totalActions: number
  totalFileChanges: number
  build: Build
  allTargets: Map<string, number>
}) {
  const {nodes, build} = props

  const nodeMap: Map<number, DisplayNode> = new Map()
  for (const [k, node] of nodes) {
    nodeMap.set(k, {...node, allowedDeps: new Map(), displayType: DisplayType.hidden})
  }

  const [colorByCfg, setColorByCfg] = useState(false)
  const [showLabels, setShowLabels] = useState(true)
  const [includeContaining, setIncludeContaining] = useState<string[]>([])
  const [excludeContaining, setExcludeContaining] = useState<string[]>([])
  const [highlighted, setHighlighted] = useState<string | null>(null)

  let nodeCounter = 0
  let totalActionsAffectedByFileChanges = 0
  let totalTargetsWithActionsThatRan = 0
  // Apply filters
  for (const [k, node] of nodeMap) {
    const target = build.targets(k)!
    const label = target.label()!.targetLabel()!

    // Targets with actions ran
    if (target.actionsLength() > 0) {
      totalTargetsWithActionsThatRan += 1
      node.displayType = DisplayType.actionsRan
    }

    if (target.changedFilesLength() > 0) {
      node.displayType = DisplayType.changedFiles
    }

    // Add highlighted
    if (highlighted) {
      if (label.includes(highlighted)) {
        node.displayType = DisplayType.highlighted
      }
    }

    // Including means we can hide everything that doesn't match the filter
    if (includeContaining.length > 0) {
      let contains = false
      for (const v of includeContaining) {
        if (label.includes(v)) {
          contains = true
          break
        }
      }
      if (!contains) {
        node.displayType = DisplayType.hidden
      }
    }

    // Excluding can hide everything except root node
    for (const v of excludeContaining) {
      if (label.includes(v)) {
        node.displayType = DisplayType.hidden
      }
    }

    // Prevent graph from having too many nodes
    if (node.displayType != DisplayType.hidden) {
      nodeCounter += 1
    }
    if (nodeCounter > MAX_NODES) {
      node.displayType = DisplayType.hidden
    }
  }

  // Always set root node
  nodeMap.get(0)!.displayType = DisplayType.rootNode

  let displayNodes: Map<number, DisplayNode> = new Map()
  let filteredNodes = new Map()
  for (const [k, node] of nodeMap) {
    if (showNode(node)) {
      filteredNodes.set(k, node)
    }
  }

  // For each node A that goes, traverse the graph bottom up BFS
  // until another node that goes is found, then add node A as allowedDep
  // Also stores shortest path length from last allowed to later add as edge label

  for (const [k, _] of filteredNodes) {
    let visited: Map<number, number> = new Map()
    visited.set(k, 0)
    let stack = [k]

    while (stack.length > 0) {
      const n1 = stack.shift()

      for (const r of nodeMap.get(n1)!.rdeps) {
        if (visited.has(r)) {
          continue
        }
        const distance = visited.get(n1)! + 1
        visited.set(r, distance)
        if (showNode(nodeMap.get(r)!)) {
          nodeMap.get(r)!.allowedDeps.set(k, distance)
        } else {
          stack.push(r)
        }
      }
    }
  }

  // Build graph in a format that the graph library understands
  const data: NodeObject[] = []
  const edges: LinkObject[] = []

  for (const [k, node] of filteredNodes) {
    const target = build.targets(k)!

    // Add nodes to graph
    data.push({
      val: 0.5,
      id: k,
      name: formatTargetLabel(target.label()!),
      color: colorByCfg ? undefined : displayTypeColors[node.displayType],
      cfg: target.label()!.cfg()!,
    })
  }

  for (const [k, node] of filteredNodes) {
    // Add edges
    for (const [d, counter] of node.allowedDeps) {
      if (!filteredNodes.has(d)) {
        throw Error("this shouldn't be possible")
      }
      edges.push({
        source: k,
        target: d,
        name: `steps: ${counter}`,
        color: 'rgba(20, 20, 20, 0.5)',
      })
    }
  }

  function applyFilters() {
    const inputValue = (id: string) =>
      (document.getElementById(id) as HTMLInputElement).value.trim()

    // Include exclude by label
    const inc = inputValue('includeContaining')
    setIncludeContaining(inc ? inc.split(',') : [])
    const exc = inputValue('excludeContaining')
    setExcludeContaining(exc ? exc.split(',') : [])

    // Highlight by label
    setHighlighted(inputValue('highlightNode'))
  }

  return (
    <>
      <div className="grid mt-4">
        <article className="message cell is-primary">
          <div className="message-header">
            <p>Build stats</p>
          </div>
          <div className="message-body">
            Nodes shown: {data.length} <br />
            Edges shown: {edges.length} <br />
            Targets with executed actions: {totalTargetsWithActionsThatRan} <br />
            Total executed actions: {props.totalActions} <br />
            Total executed actions affected by file changes: {
              totalActionsAffectedByFileChanges
            }{' '}
            <br />
            Number of files with changes: {props.totalFileChanges}
          </div>
        </article>
        <article className="message cell is-info">
          <div className="message-header">
            <p>Node colors</p>
          </div>
          <div className="message-body">
            Green: node with changed files <br />
            Blue: node with actions that ran <br />
            Purple: highlighted node via filter <br />
          </div>
        </article>
        <div className="cell">
          <div className="field">
            <label className="label is-size-7">Filter by labels:</label>
            <div className="control">
              <input
                id="includeContaining"
                className="input is-small"
                type="text"
                placeholder="Only include containing"
              />
            </div>
            <div className="control">
              <input
                id="excludeContaining"
                className="input is-small"
                type="text"
                placeholder="Exclude containing"
              />
            </div>
          </div>
        </div>
        <div className="cell">
          <div className="field">
            <label className="label is-size-7">Highlight target by label:</label>
            <div className="control">
              <input
                id="highlightNode"
                className="input is-small"
                type="text"
                placeholder="Label to highlight"
              />
            </div>
          </div>
          <div className="cell">
            <button type="submit" onClick={applyFilters} className="button is-dark is-small">
              <span>Apply filters</span>
            </button>
          </div>
        </div>
        <div className="cell">
          <label className="checkbox ml-2 mt-4">
            <input
              type="checkbox"
              checked={colorByCfg}
              onChange={e => setColorByCfg(e.target.checked)}></input>{' '}
            Color by configuration
          </label>
          <label className="checkbox ml-2 mt-4">
            <input
              type="checkbox"
              checked={showLabels}
              onChange={e => setShowLabels(e.target.checked)}></input>{' '}
            Show labels
          </label>
        </div>
      </div>
      <h3 className="title is-3">Showing targets with executed actions</h3>
      <GraphViz2
        nodes={data}
        colorByCfg={colorByCfg}
        showLabels={showLabels}
        links={edges}
        openTarget={(name: string) => {
          const url = new URL(window.location.href)
          url.searchParams.set('target', name)
          url.searchParams.delete('graph')
          window.open(url.toString(), '_blank')
        }}
      />
    </>
  )
}
