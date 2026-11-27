use super::{is_image, DeckError, Slide, SlideImage, TextShape};
use roxmltree::{Document, Node, NodeId};
use std::collections::BTreeMap;
use std::fs::File;
use std::io::Read;
use std::path::Path;
use std::sync::Arc;
use zip::ZipArchive;

type Archive = ZipArchive<File>;

fn part_bytes(archive: &mut Archive, part: &str) -> Result<Vec<u8>, DeckError> {
    let mut bytes = Vec::new();
    archive.by_name(part)?.read_to_end(&mut bytes)?;
    Ok(bytes)
}

fn xml(archive: &mut Archive, part: &str) -> Result<String, DeckError> {
    let bytes = part_bytes(archive, part)?;
    let (encoding, skip) =
        encoding_rs::Encoding::for_bom(&bytes).unwrap_or((encoding_rs::UTF_8, 0));
    encoding
        .decode_without_bom_handling_and_without_replacement(&bytes[skip..])
        .map(|text| text.into_owned())
        .ok_or_else(|| DeckError::Invalid(format!("Invalid XML encoding in {part}")))
}

// Relationship targets use package URIs even on Windows. Resolve inside the archive.
fn resolve_part(archive: &Archive, owner: &str, target: &str) -> Result<String, DeckError> {
    let mut parts: Vec<_> = if target.starts_with('/') {
        Vec::new()
    } else {
        owner
            .rsplit_once('/')
            .map_or_else(Vec::new, |(dir, _)| dir.split('/').collect())
    };
    for part in target.split('/') {
        match part {
            "" | "." => {}
            ".." => {
                parts.pop().ok_or_else(|| {
                    DeckError::Invalid(format!("Invalid relationship target {target}"))
                })?;
            }
            _ => parts.push(part),
        }
    }
    let name = parts.join("/");
    if !name.contains('%') || archive.index_for_name(&name).is_some() {
        return Ok(name);
    }
    percent_encoding::percent_decode_str(&name)
        .decode_utf8()
        .map(|decoded| decoded.into_owned())
        .map_err(|e| DeckError::Invalid(e.to_string()))
}

struct Relationship {
    target: String,
    kind: String,
}

fn relationships(
    archive: &mut Archive,
    owner: &str,
) -> Result<BTreeMap<String, Relationship>, DeckError> {
    let (dir, name) = owner.rsplit_once('/').unwrap_or(("", owner));
    let part = format!("{dir}/_rels/{name}.rels");
    let mut relationships = BTreeMap::new();
    if archive.index_for_name(&part).is_none() {
        return Ok(relationships);
    }
    let text = xml(archive, &part)?;
    let doc = Document::parse(&text)?;
    for node in doc
        .root_element()
        .children()
        .filter(|n| n.tag_name().name() == "Relationship")
    {
        if node
            .attribute("TargetMode")
            .is_some_and(|s| s.eq_ignore_ascii_case("External"))
        {
            continue;
        }
        if let (Some(id), Some(target)) = (node.attribute("Id"), node.attribute("Target")) {
            let kind = node
                .attribute("Type")
                .unwrap_or("")
                .rsplit('/')
                .next()
                .unwrap_or("");
            relationships.insert(
                id.into(),
                Relationship {
                    target: resolve_part(archive, owner, target)?,
                    kind: kind.into(),
                },
            );
        }
    }
    Ok(relationships)
}

fn rel_id<'a>(node: Node<'a, '_>, name: &str) -> Option<&'a str> {
    node.attributes()
        .find(|attr| {
            attr.name() == name
                && attr
                    .namespace()
                    .is_some_and(|ns| ns.ends_with("/relationships"))
        })
        .map(|attr| attr.value())
}

fn slide_parts(archive: &mut Archive) -> Result<Vec<String>, DeckError> {
    let owner = "ppt/presentation.xml";
    if archive.index_for_name(owner).is_some()
        && archive
            .index_for_name("ppt/_rels/presentation.xml.rels")
            .is_some()
    {
        let text = xml(archive, owner)?;
        let doc = Document::parse(&text)?;
        let rels = relationships(archive, owner)?;
        if let Some(list) = doc
            .root_element()
            .children()
            .find(|n| n.tag_name().name() == "sldIdLst")
        {
            return list
                .children()
                .filter(|n| n.tag_name().name() == "sldId")
                .map(|n| {
                    rel_id(n, "id")
                        .and_then(|id| rels.get(id))
                        .filter(|r| r.kind == "slide" || r.kind.is_empty())
                        .map(|r| r.target.clone())
                        .ok_or_else(|| {
                            DeckError::Invalid(
                                "Missing slide relationship in presentation.xml".into(),
                            )
                        })
                })
                .collect();
        }
    }
    let mut numbered: Vec<_> = archive
        .file_names()
        .filter_map(|name| {
            let number = name
                .strip_prefix("ppt/slides/slide")?
                .strip_suffix(".xml")?
                .parse::<u32>()
                .ok()?;
            Some((number, name.to_owned()))
        })
        .collect();
    numbered.sort();
    Ok(numbered.into_iter().map(|(_, name)| name).collect())
}

fn drawing(node: Node<'_, '_>, name: &str) -> bool {
    node.tag_name().name() == name
        && node.tag_name().namespace().is_some_and(|ns| {
            ns.ends_with("/drawingml/2006/main") || ns.ends_with("/drawingml/main")
        })
}

fn text_shapes(doc: &Document<'_>) -> Vec<TextShape> {
    let mut shapes: Vec<TextShape> = Vec::new();
    let mut previous: Option<NodeId> = None;
    for paragraph in doc.descendants().filter(|&n| drawing(n, "p")) {
        let shape = paragraph
            .ancestors()
            .find(|n| matches!(n.tag_name().name(), "sp" | "graphicFrame" | "cxnSp"))
            .unwrap_or_else(|| doc.root_element());
        if previous != Some(shape.id()) {
            let name = shape
                .descendants()
                .find(|n| n.tag_name().name() == "cNvPr")
                .and_then(|n| n.attribute("name"));
            shapes.push(TextShape {
                name: name.unwrap_or("").into(),
                paragraphs: Vec::new(),
            });
            previous = Some(shape.id());
        }
        let runs = paragraph
            .descendants()
            .filter_map(|n| {
                if drawing(n, "t") {
                    Some(n.text().unwrap_or("").to_owned())
                } else if drawing(n, "br") {
                    Some("\n".into())
                } else {
                    None
                }
            })
            .collect();
        shapes.last_mut().unwrap().paragraphs.push(runs);
    }
    shapes
}

struct Package {
    archive: Archive,
    media: BTreeMap<String, Arc<[u8]>>,
}

impl Package {
    fn images(
        &mut self,
        node: Node<'_, '_>,
        rels: &BTreeMap<String, Relationship>,
    ) -> Result<Vec<SlideImage>, DeckError> {
        let mut images = Vec::new();
        for blip in node.descendants().filter(|&n| drawing(n, "blip")) {
            let Some(rel) = rel_id(blip, "embed").and_then(|id| rels.get(id)) else {
                continue;
            };
            if !(rel.kind.is_empty() || rel.kind == "image") || !is_image(Path::new(&rel.target)) {
                continue;
            }
            let bytes = match self.media.entry(rel.target.clone()) {
                std::collections::btree_map::Entry::Occupied(entry) => entry.into_mut(),
                std::collections::btree_map::Entry::Vacant(entry) => {
                    entry.insert(part_bytes(&mut self.archive, &rel.target)?.into())
                }
            };
            images.push(SlideImage {
                source: rel.target.clone(),
                bytes: bytes.clone(),
                is_background: blip.ancestors().any(|n| n.tag_name().name() == "bg"),
            });
        }
        Ok(images)
    }

    fn slide(&mut self, part: &str) -> Result<Slide, DeckError> {
        let text = xml(&mut self.archive, part)?;
        let doc = Document::parse(&text)?;
        let mut rels = relationships(&mut self.archive, part)?;
        let mut images = self.images(doc.root_element(), &rels)?;

        // The first explicit background in slide, layout, master order wins, including a solid fill.
        if !doc.descendants().any(|n| n.tag_name().name() == "bg") {
            for kind in ["slideLayout", "slideMaster"] {
                let Some(parent) = rels.values().find(|r| r.kind == kind) else {
                    break;
                };
                let parent_xml = xml(&mut self.archive, &parent.target)?;
                let parent_doc = Document::parse(&parent_xml)?;
                rels = relationships(&mut self.archive, &parent.target)?;
                if let Some(bg) = parent_doc
                    .descendants()
                    .find(|n| n.tag_name().name() == "bg")
                {
                    let mut background = self.images(bg, &rels)?;
                    background.append(&mut images);
                    images = background;
                    break;
                }
            }
        }
        Ok(Slide {
            shapes: text_shapes(&doc),
            images,
        })
    }
}

pub(super) fn load(path: &Path) -> Result<Vec<Slide>, DeckError> {
    let mut package = Package {
        archive: Archive::new(File::open(path)?)?,
        media: BTreeMap::new(),
    };
    slide_parts(&mut package.archive)?
        .into_iter()
        .map(|part| package.slide(&part))
        .collect()
}
