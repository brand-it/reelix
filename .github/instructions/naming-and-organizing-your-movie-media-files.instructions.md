---
applyTo: '**'
---
Naming and organizing your Movie files
The scanners and metadata agents used by Plex will work best when your major types of content are separated from each other. We strongly recommend separating movie and television content into separate main directories. For instance, you might use something like this:
/Media
   /Movies
      movie content
   /Music
      music content
   /TV Shows
      television content
Warning!: Plex will do its best to appropriately find and match content. However, a failure to separate content such as movies and TV shows may result in unexpected or incorrect behavior.

In the above example, it is the main folder of each type of content (e.g. /Movies, /Music, /TV Shows) that you would specify as the content location for that library type.
You might have your own image files for movie posters & backgrounds, subtitles, your own movie “extras”, etc. To use these, ensure they are named and organized, and that the Local Media Assets source is enabled and ordered correctly.
Related Page: Local Media Assets - Movies
Note: We use .ext as a generic file extension in the naming/organizing instructions. You should use appropriate file extension for your files, of course. (Some operating systems such as Windows may hide your file extensions by default.)

Movies in Their Own Folders
Movie files can be placed into individual folders and this is the recommended method, as it can (sometimes significantly) increase the speed of scanning in new media. This method is also useful in cases where you have external media for a movie (e.g. custom poster, external subtitle files, etc.). Name the folder the same as the movie naming:
/Movies/MovieName (release year)/MovieName (release year).ext
/Movies
   /Avatar (2009)
      Avatar (2009).mkv
   /Batman Begins (2005)
      Batman Begins (2005).mp4
      Batman Begins (2005).en.srt
      poster.jpg
So long as you’re using the current “Plex Movie” metadata agent for the library, you can also include the IMDb or TheMovieDB ID number in curly braces to help match the movie. It must follow the form {[source]-[id]}.
/Movies
   /Batman Begins (2005) {imdb-tt0372784}
      Batman Begins (2005) {imdb-tt0372784}.mp4
/Movies
   /Batman Begins (2005) {tmdb-272}
      Batman Begins (2005) {tmdb-272}.mp4
Related Page: Using Subtitles
Related Page: Local Media Assets - Movies
Stand-Alone Movie Files
If you wish, you can also put movie files next to each other in a main folder. While this is supported, we do still recommend having the movies in individual folders as outlined above.
The structure isn’t particularly important unless you have local media assets like posters or subtitles for a particular movie (in which you should have the movie and assets together in an individual folder, as outlined above). To correctly name a movie file, name it as follows:
MovieName (release year).ext
/Movies
   Avatar (2009).mkv
   Batman Begins (2005).mp4
Multiple Editions
Tip!: The ability to specify different editions for a movie requires a Plex Pass subscription for Plex Media Server admin account.

In cases where you have a specific movie edition (Theatrical release, Director’s Cut, Extended Edition, Unrated, etc.) or even multiple editions, you can specify that edition information by appropriately naming the file and/or folder. This is only available when using the (non-“legacy”) Plex Movie agent available in Plex Media Server v1.28.1 and newer
To do so, you add the edition information inside curly braces to help distinguish the movie. It must follow the form {edition-[Edition Name]}.  You can specify whatever normal text you want for the edition name (with a max limit of 32 characters).
In cases where you group your movie files together (not within individual folders for each movie), you’ll need to make sure that the files are named appropriately.
/Movies
   Avatar (2009).mkv
   Blade Runner (1982).mp4
   Blade Runner (1982) {edition-Director's Cut}.mp4
   Blade Runner (1982) {edition-Final Cut}.mkv
   Top Gun (1986).mkv
If you are following our recommendation to use individual folders per movie, you can add the edition name to either the folder or filename or both. For consistency, we recommend including the edition information in both the folder name and the filename. This can also be used along with the {[source]-[id]} tag mentioned previously.  The order of the tags is not important.
/Movies
   /Blade Runner (1982)
      Blade Runner (1982).mp4
   /Blade Runner (1982) {edition-Director's Cut}
      Blade Runner (1982) {edition-Director's Cut}.mp4
   /Blade Runner (1982) {edition-Final Cut}
      Blade Runner (1982) {edition-Final Cut}.mkv
Note: If you’re also providing your own local trailers or extras files and you have multiple editions for the movie, you’ll want to use individual folders for each edition and then place the local extras files in whichever edition you choose to be the “main” one. (Or you could also copy the extras and place them alongside each edition.)

Related Page: Multiple Editions - Movies
Movies Split Across Multiple Files
Warning!: While Plex does have limited support for content split across multiple files, it is not the expected way to handle content. Doing this may negatively impact usage of various Plex features (including, but not limited to, preview thumbnails, chapter images, audio/subtitle stream selection across parts, and more). We recommend users instead join the files together (see below).

Movies that are split into several files (e.g. pt1, pt2), can be played back as a single item (in most, but not all, players) if named correctly. The split parts must be placed inside their own folder, named as usual for the movie. Name the files as follows:
/Movies/MovieName (release year)/MovieName (release year) – Split_Name.ext
Where Split_Name is one of the following:
cdX
discX
diskX
dvdX
partX
ptX
…and you replace X with the appropriate number (cd1, cd2, etc.).
/Movies
   /The Dark Knight (2008)
      The Dark Knight (2008) - pt1.mp4
      The Dark Knight (2008) - pt2.mp4
Notes:
Not all Plex apps support playback of stacked media
All parts must be of the same file format (e.g. all MP4 or all MKV)
All parts should have identical audio and subtitle streams in the same order
Only stacks up to 8 parts are supported
“Other Videos” libraries or those using the “Plex Video Files Scanner” do not support stacked content.
Not all features will work correctly when using “split” files.
To get a better overall experience, we strongly encourage you to instead use a tool to join/merge the individual files into a single video. There are multiple ways you can do this and a quick search in your favorite search engine should give you some options on how to “join” files. An unofficial guide with one free tool has even been posted in our forums.
Related Page: Forums: Joining multi-part movie files with MKVtoolnix GUI


Local Show and Season Extras
You can organize your local extras into specific subdirectories inside the main directory named for the TV show. Extras will be detected and used if named and stored as follows:
TV Show (Release Date)/Extra_Directory_Type/Descriptive_name.ext
Where Extra_Directory_Type is one of:
Behind The Scenes
Deleted Scenes
Featurettes
Interviews
Scenes
Shorts
Trailers
Other
Example:
/Game of Thrones
   An Interview With Emilia Clarke-interview.mkv
   /Trailers
      Trailer 1.mkv
      Trailer 2.mkv
   /Featurettes
      Special Effects.mkv
   /Season 01
      /Behind The Scenes
         A look at season 1.mkv
         Season 1 Deleted Scenes-deleted.mkv
Tip!: When extras are added after the show, season, or episode already exists in your Plex library, you will likely need to trigger the Refresh Metadata action for the show as a whole to have the new content picked up.

Local Episode Extras
Episode extras can be located alongside the episode file. They’re indicated by using specific naming at the end of the filename. Local inline extras will be detected and used if named and stored as follows:
S01E01 - Episode Title-Extra_Type.ext
Where -Extra_Type is one of:
-behindthescenes
-deleted
-featurette
-interview
-scene
-short
-trailer
-other
Note: The filename must end in the -Extra_Type value exactly. The hyphen is important and you cannot have spaces between the hyphen and word.

Examples:
Game of Thrones - S01E01 - Winter is Coming.mkv
Game of Thrones - S01E01 - Winter is Coming-deleted.mkv
Game of Thrones - S01E01 - Winter is Coming-featurette.mkv
You can add multiple-episode extras of the same type by adding a number index to the end of the suffix:
e.g.
Game of Thrones - S01E01 - Winter is Coming-deleted1.mkv
Game of Thrones - S01E01 - Winter is Coming-deleted2.mkv
Game of Thrones - S01E01 - Winter is Coming-deleted3.mkv
