---
applyTo: "**"
---

# Naming and Organizing Your TV Show Files

The scanners and metadata agents used by Plex will work best when your major types of content are separated from each other. We strongly recommend separating movie and television content into separate main directories. For instance, you might use something like this:

```
/Media
   /Movies
      movie content
   /Music
      music content
   /TV Shows
      television content
```

**Warning!** Plex will do its best to appropriately find and match content. However, a failure to separate content such as movies and TV shows may result in unexpected or incorrect behavior.

In the above example, it is the main folder of each type of content (e.g. `/Movies`, `/Music`, `/TV Shows`) that you would typically specify as the content location for that library type.

**Tip!** More specifically for television content, the folder you want to specify as the content location for the library is the folder that contains each of the individual show folders. So, if you chose to categorize your children's content separate from more "adult" content (e.g. `/TV Shows/Kids/ShowName` vs `/TV Shows/Regular/ShowName`), then you would specify `/TV Shows/Kids` as the source location for a "kids" TV library.

TV shows can be season-based, date-based, a miniseries, or more. Both the folder structure and each episode filename must be correct for the best matching experience. If you're not sure whether a show is season- or date-based, check [The Movie Database](https://www.themoviedb.org/) (TMDB) or [The TVDB](http://thetvdb.com/) and name it as it appears there.

## Content Separation

Plex requires clear separation between different content types:

- `/Media/Movies` - All movie files
- `/Media/TV Shows` - All TV show files
- `/Media/Music` - All music files

Mixing content types can cause Plex to misidentify and miscategorize your media.

## Folder Structure

Each TV show should have its own folder containing season subfolders:

```
/TV Shows/ShowName/Season XX/
```

**Important:** Always use the English word "Season" when creating season directories, even if your content is in another language.

### Show Folder Naming

For the "Plex TV Series" agent, it is recommended to always include the year alongside the series title in folder names:

```
/TV Shows/ShowName (YYYY)/
```

**Example:**

```
/TV Shows
   /Stranger Things (2016)
   /Breaking Bad (2008)
   /The Office (US) (2005)
```

### Optional Show IDs

If you are using the "Plex TV Series" agent, you can optionally include the TMDB, TVDB, or IMDb show ID in the folder name to improve matching. If you choose to do that, it must be inside curly braces:

- `ShowName (YYYY) {tmdb-123456}`
- `ShowName (YYYY) {tvdb-123456}`
- `ShowName (YYYY) {imdb-tt123456}`

## Season-Based Shows

Most television shows have episodes organized into seasons.

### Folder Structure

```
/TV Shows/ShowName (YYYY)/Season XX/
```

Season numbers are zero-padded to two digits (Season 01, Season 02, etc.).

### Episode File Naming (Single-Part Episodes)

Standard single-part episodes should be named as follows:

```
ShowName (YYYY) - sXXeYY - Episode_Title.ext
```

**IMPORTANT:** Single-part episodes should NOT have any `-pt` suffix. No `-pt1`, no part numbering at all.

**Examples:**

```
/TV Shows/Breaking Bad (2008)/Season 01/
   Breaking Bad (2008) - s01e01 - Pilot.mkv
   Breaking Bad (2008) - s01e02 - Cat's in the Bag....mkv
   Breaking Bad (2008) - s01e03 - And the Bag's in the River.mkv
```

**Format breakdown:**

- `ShowName (YYYY)` - Show title with release year
- `-` - Separator (dash)
- `sXXeYY` - Season and episode notation (s01e01 means Season 1 Episode 1)
- `-` - Separator (optional but recommended)
- `Episode_Title` - Optional episode name (optional but helpful)
- `.ext` - File extension (mkv, mp4, avi, etc.)

**Note:** It does not matter if you use dashes, dots, or just spaces as separators, but dashes are most commonly used.

## Episodes Split Across Multiple Files (Multi-Part)

**Warning!** While Plex does have limited support for content split across multiple files, it is not the expected way to handle content. Doing this may negatively impact usage of various Plex features (including, but not limited to, preview thumbnails, skip intro, audio/subtitle stream selection across parts, and more). We recommend users instead join the files together.

### Multi-Part Episode Naming

**CRITICAL RULE:** Only use `-ptX` suffixes when an episode is actually split into multiple parts. Single-part episodes should NEVER have a `-pt1` suffix.

Episodes that are split into several files (e.g., pt1, pt2) can be played back as a single item if named correctly:

```
ShowName (YYYY) - sXXeYY - Episode_Title-ptX.ext
```

Where `X` is one of the following:

- `cd1`, `cd2`, `cd3`, etc.
- `disc1`, `disc2`, `disc3`, etc.
- `disk1`, `disk2`, `disk3`, etc.
- `dvd1`, `dvd2`, `dvd3`, etc.
- `part1`, `part2`, `part3`, etc.
- `pt1`, `pt2`, `pt3`, etc.

**Examples of multi-part episodes:**

```
/TV Shows/Grey's Anatomy (2005)/Season 01/
   Grey's Anatomy (2005) - s01e01 - pt1.avi
   Grey's Anatomy (2005) - s01e01 - pt2.avi
   Grey's Anatomy (2005) - s01e02 - The First Cut is the Deepest.avi
   Grey's Anatomy (2005) - s01e03.mp4
```

**What this example shows:**

- Episode 1 is split into 2 parts → has `-pt1` and `-pt2` suffixes
- Episode 2 is a single part → has NO part suffix
- Episode 3 is a single part → has NO part suffix

### Multi-Part Episode Notes

- All parts must be of the same file format (e.g., all MP4 or all MKV)
- All parts should have identical audio and subtitle streams in the same order
- Only stacks up to 8 parts are supported
- Not all features will work correctly when using "split" files
- Not all Plex apps support playback of media split across multiple files

### Recommended Alternative

To get a better overall experience, we strongly encourage you to use a tool to join/merge the individual files into a single video. There are multiple ways you can do this and a quick search should give you some options on how to "join" files.

## Date-Based Shows

TV shows that are date-based should be named using the air date format:

```
/TV Shows/ShowName (YYYY)/Season XX/ShowName (YYYY) - YYYY-MM-DD - Optional_Info.ext
```

The date can use either the YYYY-MM-DD or DD-MM-YYYY formats and can use different separators:

- Dashes: `2011-11-15`
- Periods: `2011.11.15`
- Spaces: `2011 11 15`

**Example:**

```
/TV Shows/The Colbert Report (2005)/Season 08/
   The Colbert Report (2005) - 2011-11-15 - Elijah Wood.avi
   The Colbert Report (2005) - 2011-11-16 - Paul McCartney.avi
```

## Miniseries

A miniseries is handled just like a season-based show. You simply always use "Season 01" as the season, even if the miniseries only has one season.

**Example:**

```
/TV Shows/Band of Brothers (2001)/Season 01/
   Band of Brothers (2001) - s01e01 - Currahee.mkv
   Band of Brothers (2001) - s01e02 - Discipline.mkv
   Band of Brothers (2001) - s01e03 - Carentan.mkv
```

## Television Specials

Shows sometimes air "specials" or other content that isn't part of the standard season. "Specials" episodes are always part of season zero (i.e., season number "00") and should be placed inside a folder named either `Season 00` or `Specials`.

### Specials Folder Structure

```
/TV Shows/ShowName (YYYY)/Season 00/
   OR
/TV Shows/ShowName (YYYY)/Specials/
```

### Specials File Naming

```
ShowName (YYYY) - s00eXX - Special_Title.ext
```

**Example:**

```
/TV Shows/Grey's Anatomy (2005)/Season 00/
   Grey's Anatomy (2005) - s00e01 - Straight to the Heart.mkv
```

**Note:** If a special you have doesn't appear in TMDB (e.g., it's a DVD special, behind the scenes, goof reel, etc.), you can instead add the item as an "extra" for the show using Plex's local extras feature.

## Multiple Episodes in a Single File

If a single file covers more than one episode, name it using the following format:

```
ShowName (YYYY) - sXXeYY-eZZ - Optional_Info.ext
```

Where you specify the appropriate season, episode numbers (the first and last episode covered in the file), and file extension. For example, `s02e18-e19`.

**Example:**

```
/TV Shows/Grey's Anatomy (2005)/Season 02/
   Grey's Anatomy (2005) - s02e01-e03.avi
   Grey's Anatomy (2005) - s02e04.m4v
```

**Note:** Multi-episode files will show up individually in Plex apps when viewing your library, but playing any of the represented episodes will play the full file. If you want episodes to behave truly independently, you're best off using a tool to split the file into individual episodes.

## Complete Example

This example illustrates many of the types of content outlined previously. When creating the TV library, it is the `/TV Shows` directory that would be specified as the content location for the library.

```
/TV Shows
   /Doctor Who (1963)
      /Season 01
         Doctor Who (1963) - s01e01 - An Unearthly Child (1).mp4
         Doctor Who (1963) - s01e02 - The Cave of Skulls (2).mp4

   /From the Earth to the Moon (1998)
      /Season 01
         From the Earth to the Moon (1998) - s01e01.mp4
         From the Earth to the Moon (1998) - s01e02.mp4

   /Grey's Anatomy (2005)
      /Season 00
         Grey's Anatomy (2005) - s00e01 - Straight to the Heart.mkv
      /Season 01
         Grey's Anatomy (2005) - s01e01 - pt1.avi
         Grey's Anatomy (2005) - s01e01 - pt2.avi
         Grey's Anatomy (2005) - s01e02 - The First Cut is the Deepest.avi
         Grey's Anatomy (2005) - s01e03.mp4
      /Season 02
         Grey's Anatomy (2005) - s02e01-e03.avi
         Grey's Anatomy (2005) - s02e04.m4v

   /The Colbert Report (2005)
      /Season 08
         The Colbert Report (2005) - 2011-11-15 - Elijah Wood.avi

   /The Office (UK) (2001) {tmdb-2996}
      /Season 01
         The Office (UK) (2001) - s01e01 - Downsize.mp4

   /The Office (US) (2005) {tvdb-73244}
      /Season 01
         The Office (US) (2005) - s01e01 - Pilot.mkv
```

## Important Notes

- For the "Plex TV Series" agent, it is recommended to always include the year alongside the series title in folder and file names.
- Be sure to use the English word "Season" when creating season directories, even if your content is in another language.
- Optional info at the end of the file name is optional. If you want this info to be ignored during matching, put it in brackets, e.g., `ShowName (2020) - s01e01 - Title [1080p Bluray].mkv`.
- We use `.ext` as a generic file extension. You should use the appropriate file extension for your files.
- **Do not use `-pt1` suffix for single-part episodes**. The `-ptX` suffix is ONLY for episodes split across multiple files.
- All episode file names must include the season and episode notation (sXXeYY) for Plex to properly identify them.
- If you're unsure whether a show is season-based or date-based, check TMDB or TVDB and name it as it appears there.

## Related Resources

- [Local Files for TV Show Trailers and Extras](https://support.plex.tv/articles/local-files-for-tv-show-trailers-and-extras/)
- [The Movie Database (TMDB)](https://www.themoviedb.org/)
- [The TVDB](http://thetvdb.com/)
- [Plex Forums: Joining multi-part media files](https://forums.plex.tv/)
