import { Request, Response } from 'express';
import sharp from 'sharp';

export const getPlaceholderImage = async (req: Request, res: Response) => {
  try {
    const width = parseInt(req.params.width, 10) || 400;
    const height = parseInt(req.params.height, 10) || 300;
    const color = '#1a1a1a'; // A dark color that fits the luxury theme
    const textColor = '#555555';

    // Create a simple, elegant SVG placeholder
    const svgImage = `
      <svg width="${width}" height="${height}" viewBox="0 0 ${width} ${height}" xmlns="http://www.w3.org/2000/svg">
        <rect x="0" y="0" width="${width}" height="${height}" fill="${color}" />
        <text
          x="50%"
          y="50%"
          font-family="-apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, Helvetica, Arial, sans-serif, 'Apple Color Emoji', 'Segoe UI Emoji', 'Segoe UI Symbol'"
          font-size="20"
          dy=".3em"
          fill="${textColor}"
          text-anchor="middle">
          HeSocial
        </text>
      </svg>
    `;

    const pngBuffer = await sharp(Buffer.from(svgImage)).toFormat('png').toBuffer();

    res.set('Content-Type', 'image/png');
    res.send(pngBuffer);
  } catch (error) {
    console.error('Error generating placeholder image:', error);
    res.status(500).send('Error generating image');
  }
};
