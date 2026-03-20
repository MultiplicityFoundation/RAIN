extends Node2D
class_name AgentAvatar

## NES-style pixel-art avatar renderer.
## All sprites are generated procedurally at authentic NES resolution (8×8 px per part).
## Godot's nearest-neighbor upscaling + pixel-snap in project.godot gives the chunky
## retro look with zero external assets.

## NES DMC palette — 8 iconic colors (no dithering needed at this scale)
const PAL := {
	0: null,                         # transparent
	1: Color("#5a4a38"),              # outline / dark brown
	2: Color("#bca078"),              # skin mid
	3: Color("#f0d8a8"),              # skin light / highlight
	4: Color("#1b2230"),              # outline / deep shadow
	5: Color("#3a6abc"),              # body blue
	6: Color("#1e3a8a"),              # body dark blue
	7: Color("#7ec8e3"),              # eye / water blue
	8: Color("#f0f0e8"),              # white
	9: Color("#e82020"),               # red / mouth
	10: Color("#50a050"),             # green
	11: Color("#2860a0"),             # blue
	12: Color("#d89878"),             # skin dark / cheek
	13: Color("#a07050"),             # hair brown
	14: Color("#e8c040"),             # gold / star
	15: Color("#303030"),             # near-black
}

## Avatar types — each is a small 2-D array of PAL indices.
## Layout per type: rows are head(4), brow(1), eyes(2), mouth(1), body(6), legs(6) = 20 px tall
const AVATARS := {
	"james": {
		"palette": [1, 2, 3, 12, 7, 5, 6, 8, 9, 10, 11],
		"sprite": [
			# head outline (row 0-1)
			[0,0,1,1,1,1,0,0],
			[0,1,2,2,2,2,1,0],
			# brow
			[1,3,3,3,3,3,3,1],
			# eyes row 1
			[1,2,8,8,8,8,2,1],
			# eyes row 2
			[1,2,8,7,7,8,2,1],
			# mouth row
			[1,2,2,9,9,2,2,1],
			# body row 0
			[0,1,5,5,5,5,1,0],
			# body row 1
			[1,1,5,5,5,5,1,1],
			# body row 2
			[1,6,5,1,1,5,6,1],
			# body row 3
			[1,5,5,5,5,5,5,1],
			# legs row 0
			[0,1,1,8,8,1,1,0],
			# legs row 1
			[0,1,6,6,6,6,1,0],
			# legs row 2
			[0,1,6,1,1,6,1,0],
			# legs row 3
			[0,1,1,1,1,1,1,0],
			[0,0,1,0,0,1,0,0],
			[0,0,1,0,0,1,0,0],
			[0,0,1,1,1,1,0,0],
			[0,0,0,0,0,0,0,0],
			[0,0,0,0,0,0,0,0],
			[0,0,0,0,0,0,0,0],
		],
	},
	"jasmine": {
		"palette": [1, 2, 3, 12, 7, 10, 6, 8, 9, 11, 13],
		"sprite": [
			[0,0,1,1,1,1,0,0],
			[0,1,2,2,2,2,1,0],
			[1,3,3,3,3,3,3,1],
			[1,2,8,8,8,8,2,1],
			[1,2,8,7,7,8,2,1],
			[1,2,2,9,9,2,2,1],
			[0,1,10,10,10,10,1,0],
			[1,1,10,10,10,10,1,1],
			[1,6,10,1,1,10,6,1],
			[1,10,10,10,10,10,10,1],
			[0,1,1,8,8,1,1,0],
			[0,1,6,6,6,6,1,0],
			[0,1,6,1,1,6,1,0],
			[0,1,1,1,1,1,1,0],
			[0,0,1,0,0,1,0,0],
			[0,0,1,0,0,1,0,0],
			[0,0,1,1,1,1,0,0],
			[0,0,0,0,0,0,0,0],
			[0,0,0,0,0,0,0,0],
			[0,0,0,0,0,0,0,0],
		],
	},
	"luca": {
		"palette": [1, 2, 3, 12, 7, 15, 6, 8, 9, 10, 11],
		"sprite": [
			[0,0,1,1,1,1,0,0],
			[0,1,2,2,2,2,1,0],
			[1,3,3,3,3,3,3,1],
			[1,2,8,8,8,8,2,1],
			[1,2,8,7,7,8,2,1],
			[1,2,2,9,9,2,2,1],
			[0,1,15,15,15,15,1,0],
			[1,1,15,15,15,15,1,1],
			[1,6,15,1,1,15,6,1],
			[1,15,15,15,15,15,15,1],
			[0,1,1,8,8,1,1,0],
			[0,1,6,6,6,6,1,0],
			[0,1,6,1,1,6,1,0],
			[0,1,1,1,1,1,1,0],
			[0,0,1,0,0,1,0,0],
			[0,0,1,0,0,1,0,0],
			[0,0,1,1,1,1,0,0],
			[0,0,0,0,0,0,0,0],
			[0,0,0,0,0,0,0,0],
			[0,0,0,0,0,0,0,0],
		],
	},
	"creature": {
		"palette": [1, 2, 3, 12, 7, 10, 6, 8, 9],
		"sprite": [
			[0,0,1,1,1,1,0,0],
			[0,1,2,2,2,2,1,0],
			[1,3,3,3,3,3,3,1],
			[1,2,8,7,7,8,2,1],
			[1,2,7,7,7,7,2,1],
			[1,2,2,9,9,2,2,1],
			[0,1,10,10,10,10,1,0],
			[1,1,10,10,10,10,1,1],
			[1,6,10,1,1,10,6,1],
			[1,10,10,10,10,10,10,1],
			[0,0,1,1,1,1,0,0],
			[0,1,6,6,6,6,1,0],
			[1,1,6,6,6,6,1,1],
			[0,1,1,0,0,1,1,0],
			[0,0,1,0,0,1,0,0],
			[0,0,1,0,0,1,0,0],
			[0,0,1,1,1,1,0,0],
			[0,0,0,0,0,0,0,0],
			[0,0,0,0,0,0,0,0],
			[0,0,0,0,0,0,0,0],
		],
	},
}

const SPRITE_SCALE := 4.0   # 8 px × 4 = 32 px tall per part → full avatar ~128 px
const FRAME_HZ    := 6.0    # animation frame rate

var agent_id: String = ""
var display_name: String = ""
var style: Dictionary = {}

var _avatar_key := "james"
var _is_talking := false
var _is_active  := false
var _frame: float = 0.0

@onready var _sprite: Sprite2D = Sprite2D.new()
@onready var _talk_sprite: Sprite2D = Sprite2D.new()
@onready var _active_sprite: Sprite2D = Sprite2D.new()
@onready var _anim_sprite: Sprite2D = Sprite2D.new()

var _idle_frames: Array[Image] = []
var _talk_frames: Array[Image] = []


func _init() -> void:
	add_child(_sprite)
	add_child(_talk_sprite)
	add_child(_active_sprite)
	add_child(_anim_sprite)


func _ready() -> void:
	_sprite.centered = false
	_talk_sprite.centered = false
	_active_sprite.centered = false
	_anim_sprite.centered = false

	_sprite.position = Vector2i.ZERO
	_talk_sprite.position = Vector2i.ZERO
	_talk_sprite.visible = false
	_active_sprite.position = Vector2i.ZERO
	_anim_sprite.position = Vector2i.ZERO

	_generate_avatar_frames()
	_apply_style(style)
	_apply_active_state()


func _process(delta: float) -> void:
	if _is_talking:
		_frame += delta * FRAME_HZ
	else:
		_frame += delta * 2.0   # slower idle cycle

	var idx := int(_frame) % (_idle_frames.size() if not _is_talking else _talk_frames.size())
	_anim_sprite.texture = ImageTexture.create_from_image(
		_talk_frames[idx] if _is_talking else _idle_frames[idx]
	)
	_anim_sprite.texture.filter_mode = Texture.FILTER_NEAREST
	_anim_sprite.texture.recreate()


func configure(agent_key: String, shown_name: String, style_cfg: Dictionary) -> void:
	agent_id = agent_key
	display_name = shown_name
	_avatar_key = _pick_avatar_key(agent_key)
	_frame = 0.0
	if style_cfg:
		apply_style(style_cfg)
	_generate_avatar_frames()
	_apply_style(style)


func apply_style(style_cfg: Dictionary) -> void:
	style = style_cfg.duplicate(true)
	if is_inside_tree():
		_generate_avatar_frames()
		_apply_style(style)


func set_talking(value: bool) -> void:
	if _is_talking == value:
		return
	_is_talking = value
	_anim_sprite.visible = not value
	_talk_sprite.visible = value
	_apply_tone_to_mouth()


func set_active(value: bool) -> void:
	if _is_active == value:
		return
	_is_active = value
	_apply_active_state()


func set_tone(tone: String) -> void:
	if is_inside_tree():
		_apply_tone_to_mouth()


## ------------------------------------------------------------------
## private
## ------------------------------------------------------------------

func _pick_avatar_key(key: String) -> String:
	var lower := key.to_lower()
	for k in AVATARS.keys():
		if lower.contains(k):
			return k
	# Fall back based on hash so same agent always gets same type
	var h := abs(hash(key)) % AVATARS.size()
	return AVATARS.keys()[h]


func _generate_avatar_frames() -> void:
	_idle_frames.clear()
	_talk_frames.clear()

	var data: Array = AVATARS.get(_avatar_key, AVATARS["james"]).sprite
	var pal: Array = AVATARS.get(_avatar_key, AVATARS["james"]).palette
	var w := data[0].size()
	var h := data.size()

	# Build palette lookup from type-index → actual Color (with style overrides)
	var lookup := _build_palette_lookup(pal)

	# Idle: 2 frames (no mouth animation)
	_idle_frames.append(_build_frame(data, w, h, lookup, 0))
	_idle_frames.append(_build_frame(data, w, h, lookup, 0))

	# Talk: 2 frames (mouth alternates open / closed)
	_talk_frames.append(_build_frame(data, w, h, lookup, 1))
	_talk_frames.append(_build_frame(data, w, h, lookup, 2))

	# Wire up anim sprite
	_anim_sprite.texture = ImageTexture.create_from_image(_idle_frames[0])
	_anim_sprite.texture.filter_mode = Texture.FILTER_NEAREST
	_anim_sprite.scale = Vector2(SPRITE_SCALE, SPRITE_SCALE)

	_talk_sprite.texture = ImageTexture.create_from_image(_talk_frames[0])
	_talk_sprite.texture.filter_mode = Texture.FILTER_NEAREST
	_talk_sprite.scale = Vector2(SPRITE_SCALE, SPRITE_SCALE)


func _build_palette_lookup(type_pal: Array) -> Dictionary:
	# Returns a map: palette-index (1-based in sprite data) → Color
	var lookup := {}
	for i in type_pal:
		var c: Color = PAL.get(i, Color.BLACK)
		# Allow style overrides by matching PAL hex
		var hex := c.to_html()
		if style.has(hex):
			c = Color(style[hex])
		lookup[i] = c
	return lookup


func _build_frame(data: Array, w: int, h: int, lookup: Dictionary, mouth_frame: int) -> Image:
	var img := Image.create(w, h, false, Image.FORMAT_RGBA8)
	img.fill(Color.TRANSPARENT)

	for row in range(h):
		var row_data: Array = data[row]
		for col in range(w):
			var pal_idx: int = row_data[col]
			if pal_idx == 0:
				continue   # transparent

			var col2 := col
			# Mouth animation: data row 5 cols 2-5 swap between closed / open
			if row == 5 and mouth_frame > 0:
				if col >= 2 and col <= 5:
					if mouth_frame == 1:   # open
						col2 = col + 2 if col <= 3 else col   # inner open
					else:                 # closed — no change
						pass

			var c: Color = lookup.get(pal_idx, Color.TRANSPARENT)
			img.set_pixel(col2, row, c)

	# Generate a second frame image for talk animation (open mouth variant)
	if mouth_frame > 0:
		var open_img := Image.create(w, h, false, Image.FORMAT_RGBA8)
		open_img.fill(Color.TRANSPARENT)
		for row in range(h):
			var row_data: Array = data[row]
			for col in range(w):
				var pal_idx: int = row_data[col]
				if pal_idx == 0:
					continue
				var c: Color = lookup.get(pal_idx, Color.TRANSPARENT)
				# Open mouth: row 5 cols 3-4 become highlight
				if row == 5 and (col == 3 or col == 4):
					c = lookup.get(3, c)   # light skin = open mouth interior
				open_img.set_pixel(col, row, c)
		return open_img

	return img


func _apply_style(style_cfg: Dictionary) -> void:
	# Rebuild with style colors applied
	_generate_avatar_frames()


func _apply_tone_to_mouth() -> void:
	# Mouth expression already baked into sprite frames; nothing extra needed.
	pass


func _apply_active_state() -> void:
	if _is_active:
		_active_sprite.visible = true
		_start_pulse()
	else:
		_active_sprite.visible = false


func _start_pulse() -> void:
	var tex := ImageTexture.create_from_image(_make_ring())
	tex.filter_mode = Texture.FILTER_NEAREST
	_active_sprite.texture = tex
	_active_sprite.scale = Vector2(SPRITE_SCALE, SPRITE_SCALE)


func _make_ring() -> Image:
	# Tiny 8×8 pulsing ring (white, 1 px ring, transparent center)
	var img := Image.create(8, 8, false, Image.FORMAT_RGBA8)
	img.fill(Color.TRANSPARENT)
	var center := 4
	var radius := 3
	for y in range(8):
		for x in range(8):
			var dx := x - center
			var dy := y - center
			var dist := sqrt(dx*dx + dy*dy)
			if abs(dist - radius) < 0.8:
				img.set_pixel(x, y, Color(1, 1, 1, 0.7))
	return img
