#let myplugin = plugin("oca_render.wasm")

#let parseOca(fileName) = {
  let bytes = read(fileName, encoding: none)
  let result_bytes = json.decode(str(myplugin.get_oca(bytes)))
  return result_bytes
}
#let base64decode(data) = {
  myplugin.decode64(bytes(data))
}
#let mapJson(js, layer) = {
  let j = json.encode(js)
  let l = json.encode(layer)
  let obj = json.decode(str(myplugin.remap_json(bytes(j), bytes(l))))
  if obj.len() == 0 {
    js
  } else {
    obj
  }
}

#let convertDate(date, fmt) = {
  if date == none or fmt == none {
    return "N/A"
  }
  str(myplugin.format_date(bytes(date), bytes(fmt)))
}

#let mappingLayer(oca) = {
  oca.overlays.find( e => e.at(1).type == "spec/overlays/attribute_mapping/1.0" )
}
#let styleLayer(oca) = {
  oca.overlays.find( e => e.at(1).type == "spec/overlays/style/1.0" )
}
#let attributeTranslation(oca, language) = {
  oca.overlays.find(e => e.at(1).type == "spec/overlays/label/1.0")
}
#let formatLayer(oca) = {
  oca.overlays.find(e => e.at(1).type == "spec/overlays/format/1.0")
}

#let resolvePath(obj, path) = {
  if path == none {
    return obj
  }
  let parts = path.split(".")
  let first = parts.first()

  let rest = parts.slice(1).join(".")
  if obj == none {
    return none
  }
  return resolvePath(obj.at(first, default: none), rest)
}

#let interpolate(text, data) = {
  str(myplugin.render(bytes(text), bytes(data)))
}

#let card(data, oca) = context{
  let baseLayer = oca.capture_base
  let mapLay = mappingLayer(oca)
  let mappingLayer = if mapLay == none { none } else { mapLay.at(1)}
  let data = if mappingLayer != none {
    mapJson(data, mappingLayer)
  } else {
    data
  }
  let attrLayer = attributeTranslation(oca, "de")
  let formatLayer = formatLayer(oca).at(1)

  let attributeTranslation = if attrLayer == none { none } else {
    attrLayer.at(1)
  }
  let style = styleLayer(oca).at(1).style_json
  let fontColor = if style.textColor == "light" { color.white } else { color.black }
  let backgroundColor = rgb( style.cardColor.bit-rshift(16).bit-and(255), style.cardColor.bit-rshift(8).bit-and(255) , style.cardColor.bit-and(255),style.cardColor.bit-rshift(24).bit-and(255) )
  let propertyCard(h: auto) = rect(width: 6cm, height: h, radius: 5pt, inset: 1em , stroke: black, fill: backgroundColor)[
    #for attr in style.orderedProperties {

      set text(fontColor)
      let val =  if mappingLayer != none {
        let mappingKey = mappingLayer.attribute_mapping.at(attr, default: attr)
        let res = resolvePath(data, mappingKey)
        if res == none {
          //try fallback to no mapping
          resolvePath(data, attr)
        } else {
          res
        }
      } else {
        resolvePath(data, attr)
      }
      let attrType = baseLayer.attributes.at(attr)

      let label = if attributeTranslation == none { attr } else {
          attributeTranslation.at("attribute_labels").at(attr, default: attr)
      }
      if attrType == "DateTime" {
        let dateFormat = formatLayer.attribute_formats.at(attr)
        if dateFormat == none {
           [*#label:* #val]
        } else {
           [*#label:* #convertDate(val, dateFormat)]
        }
      } else {
        [*#label:* #val]
      }
      parbreak()
    }
  ]
  let arg = auto
  let size = measure(propertyCard(h: arg))

  if size.height < 3.5cm {
    // set fixed to 3cm
    size = measure(propertyCard(h: 3.5cm))
    arg = 3.5cm
  }

  box(width:size.width, height: size.height, radius: 5pt, stroke: black, inset:0pt, fill: backgroundColor, clip: true)[
     #set text(fontColor)
     #if style.backgroundCard != none and style.backgroundCard != "" {

      let r = regex("data\:image/(png|jpeg|jpg);base64,")
      let data = base64decode(style.backgroundCard.replace(r, "").trim())
      place(image.decode(data, fit: "cover", width: 100%))
    }
    #pad(1em)[
    = #interpolate(style.title, json.encode(data))
    == #interpolate(style.subtitle, json.encode(data))
    ]
  ]
  pagebreak()
  propertyCard(h: arg)
}

#set text(size: 8pt, font: "Noto Sans Old")
#let oca = parseOca("style.oca")
#set page(width: auto, height: auto, margin: 1pt, fill: rgb(0,0,0,0))
#card(json("data.json"), oca)
